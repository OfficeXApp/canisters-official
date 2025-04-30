// src/core/api/permissions/directory.rs

use std::collections::{HashSet, VecDeque};

use crate::{core::{api::{internals::drive_internals::is_user_in_group, types::DirectoryIDError}, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{DriveFullFilePath, FileID, FolderID}}, disks::state::state::DISKS_BY_ID_HASHTABLE, drives::state::state::OWNER_ID, groups::{state::state::is_user_on_group, types::GroupID}, permissions::{state::{helpers::{get_directory_permission_by_id, get_directory_permission_ids_for_resource}, state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE}}, types::{BreadcrumbVisibilityPreview, DirectoryPermission, DirectoryPermissionType, PermissionGranteeID, PlaceholderPermissionGranteeID, PUBLIC_GRANTEE_ID}}}, types::UserID}, rest::directory::types::{DirectoryResourceID, DirectoryResourcePermissionFE, FilePathBreadcrumb}};


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

    let is_owner = OWNER_ID.with(|owner_id| UserID(grantee_id.to_string()) == *owner_id.borrow().get());

    if is_owner {
        return vec![
            DirectoryPermissionType::View,
            DirectoryPermissionType::Edit,
            DirectoryPermissionType::Upload,
            DirectoryPermissionType::Delete,
            DirectoryPermissionType::Invite, 
            DirectoryPermissionType::Manage, 
        ];
    }

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
                    .filter_map(|id| permissions.get(id).clone())
                    .collect::<Vec<_>>()
            })
        } else {
            Vec::new()
        }
    });

    // Process permissions outside the with() closure where we can use await
    for permission in permission_entries {
        // Skip if permission is expired or not yet active
        let current_time = (ic_cdk::api::time() / 1_000_000) as i64;
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
// Add a helper function to get permissions for a resource
pub fn preview_directory_permissions(
    resource_id: &DirectoryResourceID,
    user_id: &UserID,
) -> Vec<DirectoryResourcePermissionFE> {
    // Get permission IDs for each permission type
    let mut resource_permissions = Vec::new();
    
    // Get all permission IDs for this resource using helper function
    if let Some(permission_ids) = get_directory_permission_ids_for_resource(resource_id) {
        for permission_id in &permission_ids.permissions {
            // Get the permission details using helper function
            if let Some(permission) = get_directory_permission_by_id(permission_id) {
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
    }

    resource_permissions
}

pub fn derive_breadcrumb_visibility_previews(resource_id: DirectoryResourceID) -> Vec<String> {
    // Set up our tracking flags
    let mut public_can_view = false;
    let mut public_can_modify = false;
    let mut private_can_view = false;
    let mut private_can_modify = false;
    
    // Get current time for permission validity check
    let current_time_ms = ic_cdk::api::time() / 1_000_000; // Convert nanoseconds to milliseconds
    
    // Get permission IDs for this resource directly from the hashtable
    let permission_ids = DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
        permissions_by_resource.borrow().get(&resource_id).clone()
    });
    
    // Return empty vector if no permissions found
    if permission_ids.is_none() {
        return vec![];
    }
    
    // Check each permission
    if let Some(ids) = permission_ids {
        for permission_id in ids.permissions {
            // Get the permission details directly from the hashtable
            let permission = DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions_by_id| {
                permissions_by_id.borrow().get(&permission_id).clone()
            });
            
            if let Some(permission) = permission {
                // Check if permission is currently valid (within timeframe)
                let is_active = (permission.begin_date_ms <= 0 || permission.begin_date_ms <= current_time_ms as i64) &&
                               (permission.expiry_date_ms < 0 || permission.expiry_date_ms > current_time_ms as i64);
                
                if !is_active {
                    continue; // Skip expired or not-yet-active permissions
                }
                
                // Determine if this permission grants view or modify access
                let has_view = permission.permission_types.contains(&DirectoryPermissionType::View);
                let has_modify = permission.permission_types.contains(&DirectoryPermissionType::Upload) ||
                                 permission.permission_types.contains(&DirectoryPermissionType::Edit) ||
                                 permission.permission_types.contains(&DirectoryPermissionType::Delete) ||
                                 permission.permission_types.contains(&DirectoryPermissionType::Manage);
                
                // Update flags based on grantee type
                match &permission.granted_to {
                    PermissionGranteeID::Public => {
                        if has_view {
                            public_can_view = true;
                        }
                        if has_modify {
                            public_can_modify = true;
                        }
                    },
                    _ => { // User, Group, or PlaceholderDirectoryPermissionGrantee
                        if has_view {
                            private_can_view = true;
                        }
                        if has_modify {
                            private_can_modify = true;
                        }
                    }
                }
            }
        }
    }
    
    // Build result based on determined permissions
    let mut results = Vec::new();
    
    // Prioritize modify over view (as modify implicitly includes view capabilities)
    // And prioritize public over private for clarity in the UI
    if public_can_modify {
        results.push(BreadcrumbVisibilityPreview::PublicModify);
    } else if public_can_view {
        results.push(BreadcrumbVisibilityPreview::PublicView);
    }
    
    if private_can_modify {
        results.push(BreadcrumbVisibilityPreview::PrivateModify);
    } else if private_can_view {
        results.push(BreadcrumbVisibilityPreview::PrivateView);
    }
    
    // convert to vec of strings
    results.iter().map(|r| r.to_string()).collect()
}

pub async fn derive_directory_breadcrumbs(
    resource_id: DirectoryResourceID,
    user_id: UserID,
) -> Vec<FilePathBreadcrumb> {
    // Create a vector to hold our breadcrumbs
    let mut breadcrumbs = VecDeque::new();

    let is_owner = OWNER_ID.with(|id| &id.borrow().get().clone() == &user_id);
    
    // Get the initial folder ID based on the resource type
    let initial_folder_id = match &resource_id {
        DirectoryResourceID::File(file_id) => {
            // For files, we need to get the parent folder first
            let file_metadata = match file_uuid_to_metadata.get(file_id) {
                Some(metadata) => metadata,
                None => return Vec::new(), // File not found, return empty breadcrumbs
            };
            
            // Check if user has permission to view the file
            let file_permissions = check_directory_permissions(
                resource_id.clone(),
                PermissionGranteeID::User(user_id.clone()),
            ).await;
            
            if !file_permissions.contains(&DirectoryPermissionType::View) && !is_owner {
                return Vec::new(); // User doesn't have permission to view this file
            }
            
            // If this file has sovereign permissions, we only include its parent folder and itself
            if file_metadata.has_sovereign_permissions {
                // Check permission for parent folder
                let parent_resource = DirectoryResourceID::Folder(file_metadata.parent_folder_uuid.clone());
                let parent_permissions = check_directory_permissions(
                    parent_resource.clone(),
                    PermissionGranteeID::User(user_id.clone()),
                ).await;
                
                if parent_permissions.contains(&DirectoryPermissionType::View) || is_owner {
                    // Add the parent folder only if user has permission
                    if let Some(folder_metadata) = folder_uuid_to_metadata.get(&file_metadata.parent_folder_uuid) {
                        if (folder_metadata.full_directory_path == DriveFullFilePath(format!("{}::/", folder_metadata.disk_id.to_string()))) {
                            let disk = DISKS_BY_ID_HASHTABLE.with(|map| map.borrow().get(&folder_metadata.disk_id).map(|d| d.clone()));
                            if let Some(disk) = disk {
                                breadcrumbs.push_front(FilePathBreadcrumb {
                                    resource_id: folder_metadata.id.clone().to_string(),
                                    resource_name: disk.name.clone(),
                                    visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(folder_metadata.id.clone()))
                                });
                            }
                        } else {
                            breadcrumbs.push_front(FilePathBreadcrumb {
                                resource_id: folder_metadata.id.clone().to_string(),
                                resource_name: folder_metadata.name.clone(),
                                visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(folder_metadata.id.clone()))
                            });
                        }
                        
                    }
                }

                // Add the file itself
                breadcrumbs.push_front(FilePathBreadcrumb {
                    resource_id: file_metadata.id.clone().to_string(),
                    resource_name: file_metadata.name.clone(),
                    visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::File(file_metadata.id.clone()))
                });

                return breadcrumbs.into();
            }

            // Add the file itself
            breadcrumbs.push_front(FilePathBreadcrumb {
                resource_id: file_metadata.id.clone().to_string(),
                resource_name: file_metadata.name.clone(),
                visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::File(file_metadata.id.clone()))
            });
            
            Some(file_metadata.parent_folder_uuid.clone())
        },
        DirectoryResourceID::Folder(folder_id) => {
            // For folders, include the current folder in breadcrumbs
            let folder_metadata = match folder_uuid_to_metadata.get(folder_id) {
                Some(metadata) => metadata,
                None => return Vec::new(), // Folder not found, return empty breadcrumbs
            };
            
            // Check if user has permission to view the folder
            let folder_permissions = check_directory_permissions(
                resource_id.clone(),
                PermissionGranteeID::User(user_id.clone()),
            ).await;
            
            if !folder_permissions.contains(&DirectoryPermissionType::View) && !is_owner {
                return Vec::new(); // User doesn't have permission to view this folder
            }
            
            if (folder_metadata.full_directory_path == DriveFullFilePath(format!("{}::/", folder_metadata.disk_id.to_string()))) {
                let disk = DISKS_BY_ID_HASHTABLE.with(|map| map.borrow().get(&folder_metadata.disk_id).map(|d| d.clone()));
                if let Some(disk) = disk {
                    breadcrumbs.push_front(FilePathBreadcrumb {
                        resource_id: folder_metadata.id.clone().to_string(),
                        resource_name: disk.name.clone(),
                        visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(folder_metadata.id.clone()))
                    });
                }
            } else {
                breadcrumbs.push_front(FilePathBreadcrumb {
                    resource_id: folder_metadata.id.clone().to_string(),
                    resource_name: folder_metadata.name.clone(),
                    visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(folder_metadata.id.clone()))
                });
            }
            
            // If this folder has sovereign permissions, we don't include ancestors
            if folder_metadata.has_sovereign_permissions {
                return breadcrumbs.into();
            }
            
            folder_metadata.parent_folder_uuid.clone()
        }
    };
    
    // Traverse up through parent folders to build breadcrumbs
    let mut current_folder_id = initial_folder_id;
    while let Some(folder_id) = current_folder_id {
        // Check if user has permission to view this folder
        let folder_resource = DirectoryResourceID::Folder(folder_id.clone());
        let folder_permissions = check_directory_permissions(
            folder_resource.clone(),
            PermissionGranteeID::User(user_id.clone()),
        ).await;
        
        if !folder_permissions.contains(&DirectoryPermissionType::View) && !is_owner {
            // User doesn't have permission to view this folder, stop here
            break;
        }
        
        match folder_uuid_to_metadata.get(&folder_id) {
            Some(folder_metadata) => {

                if (folder_metadata.full_directory_path == DriveFullFilePath(format!("{}::/", folder_metadata.disk_id.to_string()))) {
                    let disk = DISKS_BY_ID_HASHTABLE.with(|map| map.borrow().get(&folder_metadata.disk_id).map(|d| d.clone()));
                    if let Some(disk) = disk {
                        breadcrumbs.push_front(FilePathBreadcrumb {
                            resource_id: folder_metadata.id.clone().to_string(),
                            resource_name: disk.name.clone(),
                            visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(folder_metadata.id.clone()))
                        });
                    }
                } else {
                    breadcrumbs.push_front(FilePathBreadcrumb {
                        resource_id: folder_metadata.id.clone().to_string(),
                        resource_name: folder_metadata.name.clone(),
                        visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(folder_metadata.id.clone()))
                    });
                }
                // Stop if we've reached a folder with sovereign permissions
                if folder_metadata.has_sovereign_permissions {
                    break;
                }
                
                // Move up to the parent folder
                current_folder_id = folder_metadata.parent_folder_uuid.clone();
            },
            None => break // Folder not found, end traversal
        }
    }
    
    // Convert from VecDeque to Vec and return
    breadcrumbs.into()
}

