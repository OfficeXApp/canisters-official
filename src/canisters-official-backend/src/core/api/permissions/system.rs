// src/core/api/permissions/system.rs

use std::collections::HashSet;

use crate::core::{api::{internals::drive_internals::is_user_in_team, types::DirectoryIDError}, state::{permissions::{state::state::{SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE}, types::{PermissionGranteeID, PermissionMetadataContent, PermissionMetadataTypeEnum, PlaceholderPermissionGranteeID, SystemPermission, SystemPermissionType, SystemResourceID, SystemTableEnum, PUBLIC_GRANTEE_ID}}, teams::types::TeamID}, types::UserID};

use super::directory::parse_permission_grantee_id;


// Check if a user can CRUD the permission record
pub fn can_user_access_system_permission(
    user_id: &UserID,
    permission: &SystemPermission,
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
        PermissionGranteeID::Team(team_id) => {
            if is_user_in_team(user_id, team_id) {
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


pub fn has_system_manage_permission(user_id: &UserID, resource_id: &SystemResourceID) -> bool {
    // Use our existing check_systen_permissions which already handles inheritance
    let permissions = check_system_permissions(
        resource_id.clone(),
        PermissionGranteeID::User(user_id.clone())
    );
    permissions.contains(&SystemPermissionType::Invite)
}

// check what kind of permission a specific user has on a specific resource
pub fn check_system_permissions(
    resource_id: SystemResourceID,
    grantee_id: PermissionGranteeID,
) -> Vec<SystemPermissionType> {
    // Then check permissions for each resource and combine them
    let mut all_permissions = HashSet::new();
    let resource_permissions = check_system_resource_permissions(
        &resource_id, 
        &grantee_id,
    );
    all_permissions.extend(resource_permissions);
    all_permissions.into_iter().collect()
}


fn check_system_resource_permissions(
    resource_id: &SystemResourceID,
    grantee_id: &PermissionGranteeID,
) -> HashSet<SystemPermissionType> {
    let mut permissions_set = HashSet::new();
    
    // Get all permission IDs for this resource
    SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
        if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
            // Check each permission
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions_by_id| {
                let permissions = permissions_by_id.borrow();
                
                for permission_id in permission_ids {
                    if let Some(permission) = permissions.get(permission_id) {
                        // Skip if permission is expired or not yet active
                        let current_time = ic_cdk::api::time() as i64;
                        if permission.expiry_date_ms > 0 && permission.expiry_date_ms <= current_time {
                            continue;
                        }
                        if permission.begin_date_ms > 0 && permission.begin_date_ms > current_time {
                            continue;
                        }

                        let permission_granted_to = match parse_permission_grantee_id(&permission.granted_to.to_string()) {
                            Ok(parsed_grantee) => parsed_grantee,
                            Err(_) => continue, // Skip if parsing fails
                        };

                        // Check if permission applies to this grantee
                        let applies = match &permission_granted_to {
                            // If permission is public, anyone can access
                            PermissionGranteeID::Public => true,
                            // For other types, just match the raw IDs since we don't validate type
                            PermissionGranteeID::User(permission_user_id) => {
                                if let PermissionGranteeID::User(request_user_id) = grantee_id {
                                    permission_user_id.0 == request_user_id.0
                                } else {
                                    false
                                }
                            },
                            PermissionGranteeID::Team(permission_team_id) => {
                                if let PermissionGranteeID::Team(request_team_id) = grantee_id {
                                    permission_team_id.0 == request_team_id.0
                                } else {
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
                            // permissions_set.extend(permission.permission_types.iter().cloned());

                            // check permission metadata, handled differently for different types of permissions

                        }
                    }
                }
            });
        }
    });
    
    permissions_set
}


pub fn parse_system_resource_id(id_str: &str) -> Result<SystemResourceID, DirectoryIDError> {
    // Check if the string contains a valid prefix
    if let Some(prefix_str) = id_str.splitn(2, '_').next() {
        if prefix_str == "Table" {
            // Handle Table case - parse the remainder as SystemTableEnum
            match id_str.splitn(2, '_').nth(1) {
                Some("drives") => Ok(SystemResourceID::Table(SystemTableEnum::Drives)),
                Some("disks") => Ok(SystemResourceID::Table(SystemTableEnum::Disks)),
                Some("contacts") => Ok(SystemResourceID::Table(SystemTableEnum::Contacts)),
                Some("teams") => Ok(SystemResourceID::Table(SystemTableEnum::Teams)),
                Some("api_keys") => Ok(SystemResourceID::Table(SystemTableEnum::Api_Keys)),
                Some("permissions") => Ok(SystemResourceID::Table(SystemTableEnum::Permissions)),
                _ => Err(DirectoryIDError::InvalidPrefix),
            }
        } else {
            // Handle Record case - use the entire string
            Ok(SystemResourceID::Record(id_str.to_string()))
        }
    } else {
        Err(DirectoryIDError::MalformedID)
    }
}

// This is a helper function specifically for checking permissions table access
pub fn check_permissions_table_access(
    user_id: &UserID,
    required_permission: SystemPermissionType,
    is_owner: bool
) -> bool {
    if is_owner {
        return true;
    }

    let permissions = check_system_permissions(
        SystemResourceID::Table(SystemTableEnum::Permissions),
        PermissionGranteeID::User(user_id.clone())
    );
    permissions.contains(&required_permission)
}



pub fn check_system_resource_permissions_tags(
    resource_id: &SystemResourceID,
    grantee_id: &PermissionGranteeID,
    tag_string_value: &str,
) -> HashSet<SystemPermissionType> {
    let mut permissions_set = HashSet::new();
    
    // Get all permission IDs for this resource
    SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
        if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
            // Check each permission
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions_by_id| {
                let permissions = permissions_by_id.borrow();
                
                for permission_id in permission_ids {
                    if let Some(permission) = permissions.get(permission_id) {
                        // Skip if permission is expired or not yet active
                        let current_time = ic_cdk::api::time() as i64;
                        if permission.expiry_date_ms > 0 && permission.expiry_date_ms <= current_time {
                            continue;
                        }
                        if permission.begin_date_ms > 0 && permission.begin_date_ms > current_time {
                            continue;
                        }

                        let permission_granted_to = match parse_permission_grantee_id(&permission.granted_to.to_string()) {
                            Ok(parsed_grantee) => parsed_grantee,
                            Err(_) => continue, // Skip if parsing fails
                        };

                        // Check if permission applies to this grantee
                        let applies = match &permission_granted_to {
                            // If permission is public, anyone can access
                            PermissionGranteeID::Public => true,
                            // For other types, just match the raw IDs since we don't validate type
                            PermissionGranteeID::User(permission_user_id) => {
                                if let PermissionGranteeID::User(request_user_id) = grantee_id {
                                    permission_user_id.0 == request_user_id.0
                                } else {
                                    false
                                }
                            },
                            PermissionGranteeID::Team(permission_team_id) => {
                                if let PermissionGranteeID::Team(request_team_id) = grantee_id {
                                    permission_team_id.0 == request_team_id.0
                                } else {
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
                            // Check if there's metadata and it's a tag prefix match
                            let tag_match = match &permission.metadata {
                                Some(metadata) => {
                                    if metadata.metadata_type == PermissionMetadataTypeEnum::Tags {
                                        match &metadata.content {
                                            PermissionMetadataContent::Tags(prefix) => {
                                                // Case insensitive prefix check
                                                tag_string_value.to_lowercase()
                                                    .starts_with(&prefix.0.to_lowercase())
                                            }
                                        }
                                    } else {
                                        false
                                    }
                                },
                                None => true
                            };

                            // If there's a tag match, add the permission types
                            if tag_match {
                                permissions_set.extend(permission.permission_types.iter().cloned());
                            }
                        }
                    }
                }
            });
        }
    });
    
    permissions_set
}
