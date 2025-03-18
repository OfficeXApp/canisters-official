// src/core/api/permissions/directory.rs

use std::collections::HashSet;

use crate::{core::{api::{internals::drive_internals::is_user_in_group, types::DirectoryIDError}, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{FileID, FolderID}}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE}, types::{DirectoryPermission, DirectoryPermissionType, PermissionGranteeID, PlaceholderPermissionGranteeID, PUBLIC_GRANTEE_ID}}, groups::{state::state::is_user_on_group, types::GroupID}}, types::UserID}, rest::directory::types::{DirectoryResourceID, DirectoryResourcePermissionFE}};


// Check if a user can CRUD the permission record
pub fn can_user_access_directory_permission(
    user_id: &UserID,
    permission: &DirectoryPermission,
    is_owner: bool
) -> bool {
    // System owner can access all permissions
    if is_owner {
        return true;
    }

    // User who granted the permission can access it
    if permission.granted_by == *user_id {
        return true;
    }

    let permission_granted_to = match parse_permission_grantee_id(&permission.granted_to.to_string()) {
        Ok(parsed_grantee) => parsed_grantee,
        Err(_) => return false, // Skip if parsing fails
    };

    // Check if user is the direct grantee
    match &permission_granted_to {
        PermissionGranteeID::User(granted_user_id) => {
            if granted_user_id == user_id {
                return true;
            }
        }
        PermissionGranteeID::Group(group_id) => {
            if is_user_in_group(user_id, group_id) {
                return true;
            }
        }
        PermissionGranteeID::Public => {
            return true; // Everyone can see public permissions
        }
        PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(_) => {
            // One-time links can only be accessed by the creator
            return permission.granted_by == *user_id;
        }
    }

    false
}

// check what kind of permission a specific user has on a specific resource
pub async fn check_directory_permissions(
    resource_id: DirectoryResourceID,
    grantee_id: PermissionGranteeID,
) -> Vec<DirectoryPermissionType> {
    // First, build the list of resources to check by traversing up the hierarchy
    let resources_to_check = get_inherited_resources_list(resource_id.clone());
    
    // Then check permissions for each resource and combine them
    let mut all_permissions = HashSet::new();
    for resource in resources_to_check {
        let resource_permissions = check_directory_resource_permissions(
            &resource, 
            &grantee_id,
            resource != resource_id
        ).await;
        all_permissions.extend(resource_permissions);
    }
    
    all_permissions.into_iter().collect()
}

pub fn get_inherited_resources_list(resource_id: DirectoryResourceID) -> Vec<DirectoryResourceID> {
    let mut resources = Vec::new();

    let parsed_resource_id = match parse_directory_resource_id(&resource_id.to_string()) {
        Ok(parsed_resource) => parsed_resource,
        Err(_) => return Vec::new(), // Skip if parsing fails
    };
    
    // First check if the resource exists and get initial folder ID for traversal
    let initial_folder_id = match &parsed_resource_id {
        DirectoryResourceID::File(file_id) => {
            match file_uuid_to_metadata.get(file_id) {
                Some(file_metadata) => {
                    resources.push(resource_id.clone());
                    if file_metadata.has_sovereign_permissions {
                        return resources;
                    }
                    Some(file_metadata.parent_folder_uuid.clone())
                },
                None => return Vec::new() // File not found
            }
        },
        DirectoryResourceID::Folder(folder_id) => {
            match folder_uuid_to_metadata.get(folder_id) {
                Some(folder_metadata) => {
                    resources.push(resource_id.clone());
                    if folder_metadata.has_sovereign_permissions {
                        return resources;
                    }
                    folder_metadata.parent_folder_uuid.clone()
                },
                None => return Vec::new() // Folder not found
            }
        }
    };
    
    // Traverse up through parent folders
    let mut current_folder_id = initial_folder_id;
    while let Some(folder_id) = current_folder_id {
        match folder_uuid_to_metadata.get(&folder_id) {
            Some(folder_metadata) => {
                let folder_resource = DirectoryResourceID::Folder(folder_id.clone());
                resources.push(folder_resource);
                
                if folder_metadata.has_sovereign_permissions {
                    break;
                }
                current_folder_id = folder_metadata.parent_folder_uuid.clone();
            },
            None => break
        }
    }
    
    resources
}

async fn check_directory_resource_permissions(
    resource_id: &DirectoryResourceID,
    grantee_id: &PermissionGranteeID,
    is_parent_for_inheritance: bool,
) -> HashSet<DirectoryPermissionType> {
    let mut permissions_set = HashSet::new();
    
    // Get all permission IDs for this resource and collect them first
    let permission_entries = DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
        if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions_by_id| {
                let permissions = permissions_by_id.borrow();
                permission_ids.iter()
                    .filter_map(|id| permissions.get(id).cloned())
                    .collect::<Vec<_>>()
            })
        } else {
            Vec::new()
        }
    });

    // Process permissions outside the with() closure where we can use await
    for permission in permission_entries {
        // Skip if permission is expired or not yet active
        let current_time = ic_cdk::api::time() as i64;
        if permission.expiry_date_ms > 0 && permission.expiry_date_ms <= current_time {
            continue;
        }
        if permission.begin_date_ms > 0 && permission.begin_date_ms > current_time {
            continue;
        }
        // Skip if permission lacks inheritance and is_parent_for_inheritance
        if !permission.inheritable && is_parent_for_inheritance {
            continue;
        }

        let permission_granted_to = match parse_permission_grantee_id(&permission.granted_to.to_string()) {
            Ok(parsed_grantee) => parsed_grantee,
            Err(_) => continue, // Skip if parsing fails
        };

        // Check if permission applies to this grantee
        let applies = match &permission_granted_to {
            PermissionGranteeID::Public => true,
            PermissionGranteeID::User(permission_user_id) => {
                if let PermissionGranteeID::User(request_grantee_id) = grantee_id {
                    permission_user_id.0 == request_grantee_id.0
                } else {
                    false
                }
            },
            PermissionGranteeID::Group(permission_group_id) => {
                if let PermissionGranteeID::User(request_user_id) = grantee_id {
                    is_user_on_group(request_user_id, permission_group_id).await
                }
                else if let PermissionGranteeID::Group(request_group_id) = grantee_id {
                    permission_group_id.0 == request_group_id.0
                }
                else {
                    false
                }
            },
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(permission_link_id) => {
                if let PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(request_link_id) = grantee_id {
                    permission_link_id.0 == request_link_id.0
                } else {
                    false
                }
            }
        };

        if applies {
            permissions_set.extend(permission.permission_types.iter().cloned());
        }
    }
    
    permissions_set
}

pub async fn has_directory_manage_permission(user_id: &UserID, resource_id: &DirectoryResourceID) -> bool {
    // Use our existing check_directory_permissions which already handles inheritance
    let permissions = check_directory_permissions(
        resource_id.clone(),
        PermissionGranteeID::User(user_id.clone())
    ).await;
    permissions.contains(&DirectoryPermissionType::Invite)
}

pub fn parse_directory_resource_id(id_str: &str) -> Result<DirectoryResourceID, DirectoryIDError> {
    // Check if the string contains a valid prefix
    if let Some(prefix_str) = id_str.splitn(2, '_').next() {
        match prefix_str {
            "FileID" => Ok(DirectoryResourceID::File(FileID(id_str.to_string()))),
            "FolderID" => Ok(DirectoryResourceID::Folder(FolderID(id_str.to_string()))),
            _ => Err(DirectoryIDError::InvalidPrefix),
        }
    } else {
        Err(DirectoryIDError::MalformedID)
    }
}

pub fn parse_permission_grantee_id(id_str: &str) -> Result<PermissionGranteeID, DirectoryIDError> {
    // First check if it's the public grantee
    if id_str == PUBLIC_GRANTEE_ID {
        return Ok(PermissionGranteeID::Public);
    }

    // Check if the string contains a valid prefix
    if let Some(prefix_str) = id_str.splitn(2, '_').next() {
        match prefix_str {
            "UserID" => Ok(PermissionGranteeID::User(UserID(id_str.to_string()))),
            "GroupID" => Ok(PermissionGranteeID::Group(GroupID(id_str.to_string()))),
            "PlaceholderPermissionGranteeID" => Ok(PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(PlaceholderPermissionGranteeID(id_str.to_string()))),
            _ => Err(DirectoryIDError::InvalidPrefix),
        }
    } else {
        Err(DirectoryIDError::MalformedID)
    }
}

// Add a helper function to get permissions for a resource
pub fn preview_directory_permissions(
    resource_id: &DirectoryResourceID,
    user_id: &UserID,
) -> Vec<DirectoryResourcePermissionFE> {
    
    // Get permission IDs for each permission type
    let mut resource_permissions = Vec::new();
    
    DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
        if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions_by_id| {
                let permissions = permissions_by_id.borrow();
                
                for permission_id in permission_ids {
                    if let Some(permission) = permissions.get(permission_id) {
                        let permission_granted_to = match parse_permission_grantee_id(&permission.granted_to.to_string()) {
                            Ok(parsed_grantee) => parsed_grantee,
                            Err(_) => continue,
                        };

                        // Check if permission applies to this user
                        let applies = match &permission_granted_to {
                            PermissionGranteeID::Public => true,
                            PermissionGranteeID::User(permission_user_id) => permission_user_id == user_id,
                            PermissionGranteeID::Group(group_id) => is_user_in_group(user_id, group_id),
                            _ => false
                        };

                        if applies {
                            for grant_type in &permission.permission_types {
                                resource_permissions.push(DirectoryResourcePermissionFE {
                                    permission_id: permission_id.clone().to_string(),
                                    grant_type: grant_type.clone().to_string()
                                });
                            }
                        }
                    }
                }
            });
        }
    });

    resource_permissions
}