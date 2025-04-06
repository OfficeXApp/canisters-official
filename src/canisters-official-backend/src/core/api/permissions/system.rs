// src/core/api/permissions/system.rs

use std::collections::HashSet;

use crate::core::{api::{internals::drive_internals::is_user_in_group, types::DirectoryIDError}, state::{drives::state::state::OWNER_ID, groups::{state::state::{is_user_on_local_group, GROUPS_BY_ID_HASHTABLE, GROUPS_BY_TIME_LIST}, types::GroupID}, permissions::{state::state::{SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE}, types::{PermissionGranteeID, PermissionMetadataContent, PermissionMetadataTypeEnum, PlaceholderPermissionGranteeID, SystemPermission, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum, PUBLIC_GRANTEE_ID}}}, types::UserID};

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
    // Get a mutable HashSet to collect permissions
    let mut all_permissions = HashSet::new();
    // First, check direct permissions for the grantee
    let resource_permissions = check_system_resource_permissions(
        &resource_id, 
        &grantee_id,
    );
    all_permissions.extend(resource_permissions);

    // Always check public permissions (for any grantee type)
    let public_permissions = check_system_resource_permissions(
        &resource_id,
        &PermissionGranteeID::Public,
    );
    all_permissions.extend(public_permissions);


    // If the grantee is a user, also check group permissions
    if let PermissionGranteeID::User(user_id) = &grantee_id {
        // Check all groups the user is a member of (using GROUPS_BY_TIME_LIST)
        crate::core::state::groups::state::state::GROUPS_BY_TIME_LIST.with(|group_list| {
            for group_id in group_list.borrow().iter() {
                // Use the existing is_user_on_local_group function
                if crate::core::state::groups::state::state::is_user_on_local_group(user_id, &crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE.with(|groups| groups.borrow().get(group_id).cloned().unwrap())) {
                    // Add this group's permissions
                    let group_permissions = check_system_resource_permissions(
                        &resource_id,
                        &PermissionGranteeID::Group(group_id.clone()),
                    );
                    all_permissions.extend(group_permissions);
                }
            }
        });
    }
    
    // Convert the HashSet to a Vec and return
    all_permissions.into_iter().collect()
}


fn check_system_resource_permissions(
    resource_id: &SystemResourceID,
    grantee_id: &PermissionGranteeID,
) -> HashSet<SystemPermissionType> {
    let mut permissions_set = HashSet::new();

    // check if grantee_id is OWNER_ID, and if so then just return all permissions
    if let PermissionGranteeID::User(user_id) = grantee_id {
        let is_owner = OWNER_ID.with(|owner_id| user_id == &*owner_id.borrow());
        if is_owner {
            let mut owner_permissions = HashSet::new();
            owner_permissions.insert(SystemPermissionType::Create);
            owner_permissions.insert(SystemPermissionType::View);
            owner_permissions.insert(SystemPermissionType::Edit);
            owner_permissions.insert(SystemPermissionType::Delete);
            owner_permissions.insert(SystemPermissionType::Invite);
            return owner_permissions;
        }
    }
    
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
                            PermissionGranteeID::Group(permission_group_id) => {
                                if let PermissionGranteeID::Group(request_group_id) = grantee_id {
                                    permission_group_id.0 == request_group_id.0
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
                            permissions_set.extend(permission.permission_types.iter().cloned());
                        }
                    }
                }
            });
        }
    });
    
    permissions_set
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

pub fn check_system_resource_permissions_labels(
    resource_id: &SystemResourceID,
    grantee_id: &PermissionGranteeID,
    label_string_value: &str,
) -> HashSet<SystemPermissionType> {
    let mut permissions_set = HashSet::new();
    
    // Get direct permissions
    let direct_permissions = check_system_resource_permissions_labels_internal(
        resource_id,
        grantee_id,
        label_string_value,
    );
    permissions_set.extend(direct_permissions);

    // Always check public permissions
    permissions_set.extend(check_system_resource_permissions_labels_internal(
        resource_id,
        &PermissionGranteeID::Public,
        label_string_value,
    ));
    
    // If the grantee is a user, also check group permissions
    if let PermissionGranteeID::User(user_id) = grantee_id {
        // Check all groups the user is a member of
        GROUPS_BY_TIME_LIST.with(|group_list| {
            for group_id in group_list.borrow().iter() {
                // Use the existing is_user_on_local_group function
                if is_user_on_local_group(user_id, &GROUPS_BY_ID_HASHTABLE.with(|groups| groups.borrow().get(group_id).cloned().unwrap())) {
                    // Add this group's permissions
                    let group_permissions = check_system_resource_permissions_labels_internal(
                        resource_id,
                        &PermissionGranteeID::Group(group_id.clone()),
                        label_string_value,
                    );
                    permissions_set.extend(group_permissions);
                }
            }
        });
    }
    
    permissions_set
}


fn check_system_resource_permissions_labels_internal(
    resource_id: &SystemResourceID,
    grantee_id: &PermissionGranteeID,
    label_string_value: &str,
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
                            PermissionGranteeID::Group(permission_group_id) => {
                                if let PermissionGranteeID::Group(request_group_id) = grantee_id {
                                    permission_group_id.0 == request_group_id.0
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
                            // Check if there's metadata and it's a label prefix match
                            let label_match = match &permission.metadata {
                                Some(metadata) => {
                                    if metadata.metadata_type == PermissionMetadataTypeEnum::Labels {
                                        match &metadata.content {
                                            PermissionMetadataContent::Labels(prefix) => {
                                                // Case insensitive prefix check
                                                label_string_value.to_lowercase()
                                                    .starts_with(&prefix.0.to_lowercase())
                                            },
                                            _ => false
                                        }
                                    } else {
                                        false
                                    }
                                },
                                None => true
                            };

                            // If there's a label match, add the permission types
                            if label_match {
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
