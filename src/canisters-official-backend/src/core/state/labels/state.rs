// src/core/state/labels/state.rs

use std::cell::RefCell;
use std::collections::HashMap;

use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap, StableVec, DefaultMemoryImpl};

use crate::{
    core::{
        api::{types::DirectoryIDError, uuid::generate_uuidv4},
        state::{
            api_keys::{state::state::APIKEYS_BY_ID_HASHTABLE, types::ApiKeyID}, contacts::{state::state::CONTACTS_BY_ID_HASHTABLE, types::Contact}, directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{FileID, FolderID}}, disks::{state::state::DISKS_BY_ID_HASHTABLE, types::DiskID}, drives::{state::state::DRIVES_BY_ID_HASHTABLE, types::DriveID}, group_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::GroupInviteID}, groups::{state::state::GROUPS_BY_ID_HASHTABLE, types::GroupID}, labels::types::{LabelResourceID, LabelStringValue}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE}, types::{DirectoryPermissionID, SystemPermissionID}}, webhooks::{state::state::WEBHOOKS_BY_ID_HASHTABLE, types::WebhookID}
        },
        types::{IDPrefix, UserID}
    },
    debug_log, rest::types::ValidationError, MEMORY_MANAGER
};

use super::types::{HexColorString, Label, LabelID};


type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

// Define memory IDs for each stable structure
pub const LABELS_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(34);
pub const LABELS_BY_VALUE_MEMORY_ID: MemoryId = MemoryId::new(35);
pub const LABELS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(36);

thread_local! {
    // Convert HashMap to StableBTreeMap for labels by ID
    pub(crate) static LABELS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<LabelID, Label, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(LABELS_BY_ID_MEMORY_ID))
        )
    );
    
    // Convert HashMap to StableBTreeMap for labels by value
    pub(crate) static LABELS_BY_VALUE_HASHTABLE: RefCell<StableBTreeMap<LabelStringValue, LabelID, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(LABELS_BY_VALUE_MEMORY_ID))
        )
    );
    
    // Convert Vec to StableVec for labels by time
    pub(crate) static LABELS_BY_TIME_LIST: RefCell<StableVec<LabelID, Memory>> = RefCell::new(
        StableVec::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(LABELS_BY_TIME_MEMORY_ID))
        ).expect("Failed to initialize LABELS_BY_TIME_LIST")
    );
}


pub fn validate_uuid4_string_with_prefix(prefix_uuid_string: &str, prefix: IDPrefix) -> Result<(), ValidationError> {
    let parts: Vec<&str> = prefix_uuid_string.split('_').collect();
    // check prefix portion
    if parts.len() != 2 || parts[0] != prefix.as_str().replace("_", "") {
        return Err(ValidationError {
            field: "uuid".to_string(),
            message: format!("String must be formatted as {}_uuid", prefix.as_str()),
        });
    }
    // check uuid portion
    if parts.len() != 2 {
        return Err(ValidationError {
            field: "uuid".to_string(),
            message: "String must be formatted as prefix_uuid".to_string(),
        });
    }

    let uuid_str = parts[1];

    // Basic UUID v4 validation without external library
    let is_valid_uuid_v4 = uuid_str.len() == 36
        && uuid_str.chars().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            14 => c == '4', // UUID version 4
            19 => matches!(c, '8' | '9' | 'a' | 'b'), // UUID variant
            _ => c.is_ascii_hexdigit(),
        });

    if !is_valid_uuid_v4 {
        return Err(ValidationError {
            field: "uuid".to_string(),
            message: "Invalid UUID v4 format".to_string(),
        });
    }

    // Check if UUID has already been claimed
    crate::core::state::drives::state::state::UUID_CLAIMED.with(|claimed| {
        if claimed.borrow().contains_key(&uuid_str.to_string()) {
            Err(ValidationError {
                field: "uuid".to_string(),
                message: "UUID has already been claimed".to_string(),
            })
        } else {
            Ok(())
        }
    })
}


/// Validates a label string to ensure it meets requirements
pub fn validate_label_value(label_value: &str) -> Result<LabelStringValue, String> {
    // Check length
    if label_value.is_empty() {
        return Err("Label cannot be empty".to_string());
    }
    if label_value.len() > 64 {
        return Err("Label cannot exceed 64 characters".to_string());
    }

    // Check characters
    if !label_value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Label can only contain alphanumeric characters and underscores".to_string());
    }

    // Convert to lowercase for consistency
    Ok(LabelStringValue(label_value.to_lowercase()))
}

pub fn validate_color(color: &str) -> Result<HexColorString, String> {
    // Check length
    if color.is_empty() {
        return Err("Color cannot be empty".to_string());
    }
    if color.len() != 7 {
        return Err("Color must be a 7-character hex string".to_string());
    }

    // Check prefix
    if !color.starts_with('#') {
        return Err("Color must start with '#'".to_string());
    }

    // Check characters (excluding the # prefix)
    if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Color must be a valid hex code".to_string());
    }

    Ok(HexColorString(color.to_uppercase()))
}

/// Parse a resource ID string into the appropriate LabelResourceID enum
pub fn parse_label_resource_id(id_str: &str) -> Result<LabelResourceID, DirectoryIDError> {
    // Check if the string contains a valid prefix
    if let Some(prefix_str) = id_str.splitn(2, '_').next() {
        match prefix_str {
            "ApiKeyID" => Ok(LabelResourceID::ApiKey(ApiKeyID(id_str.to_string()))),
            "UserID" => Ok(LabelResourceID::Contact(UserID(id_str.to_string()))),
            "FileID" => Ok(LabelResourceID::File(FileID(id_str.to_string()))),
            "FolderID" => Ok(LabelResourceID::Folder(FolderID(id_str.to_string()))),
            "DiskID" => Ok(LabelResourceID::Disk(DiskID(id_str.to_string()))),
            "DriveID" => Ok(LabelResourceID::Drive(DriveID(id_str.to_string()))),
            "DirectoryPermissionID" => Ok(LabelResourceID::DirectoryPermission(DirectoryPermissionID(id_str.to_string()))),
            "SystemPermissionID" => Ok(LabelResourceID::SystemPermission(SystemPermissionID(id_str.to_string()))),
            "InviteID" => Ok(LabelResourceID::GroupInvite(GroupInviteID(id_str.to_string()))),
            "GroupID" => Ok(LabelResourceID::Group(GroupID(id_str.to_string()))),
            "WebhookID" => Ok(LabelResourceID::Webhook(WebhookID(id_str.to_string()))),
            "LabelID" => Ok(LabelResourceID::Label(LabelID(id_str.to_string()))),
            _ => Err(DirectoryIDError::InvalidPrefix),
        }
    } else {
        Err(DirectoryIDError::MalformedID)
    }
}

/// Add a label to a resource
pub fn add_label_to_resource(resource_id: &LabelResourceID, label_value: &LabelStringValue) -> Result<(), String> {
    // First, make sure the resource exists
    let resource_exists = match resource_id {
        LabelResourceID::ApiKey(id) => APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Contact(id) => CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::File(id) => file_uuid_to_metadata.contains_key(id),
        LabelResourceID::Folder(id) => folder_uuid_to_metadata.contains_key(id),
        LabelResourceID::Disk(id) => DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Drive(id) => DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::DirectoryPermission(id) => DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::SystemPermission(id) => SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::GroupInvite(id) => INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Group(id) => GROUPS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Webhook(id) => WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Label(id) => LABELS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
    };

    if !resource_exists {
        return Err(format!("Resource {:?} not found", resource_id));
    }

    // Check if the label exists, create it if not
    let label_id = LABELS_BY_VALUE_HASHTABLE.with(|store| {
        // Clone the LabelID if found to avoid lifetime issues
        if let Some(id) = store.borrow().get(label_value) {
            Some(id.clone())
        } else {
            None
        }
    }).unwrap_or_else(|| {
        let label_id = LabelID(generate_uuidv4(IDPrefix::LabelID));
        let label = Label {
            id: label_id.clone(),
            value: label_value.clone(),
            public_note: None,
            private_note: None,
            color: HexColorString("#FFFFFF".to_string()),
            created_at: ic_cdk::api::time() / 1_000_000,
            last_updated_at: ic_cdk::api::time() / 1_000_000,
            resources: vec![resource_id.clone()],
            labels: vec![],
            created_by: UserID("".to_string()),
            external_id: None,
            external_payload: None,
        };
    
        LABELS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(label_id.clone(), label);
        });
        LABELS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().insert(label_value.clone(), label_id.clone());
        });
        LABELS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(&label_id.clone());
        });
    
        label_id
    });

    // Add label to the resource
    match resource_id {
        LabelResourceID::ApiKey(id) => {
            APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    // Add labels field if not already present
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
        LabelResourceID::Contact(id) => {
            CONTACTS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
        LabelResourceID::File(id) => {
            file_uuid_to_metadata.with_mut(|files| {
                if let Some(resource) = files.get_mut(id) {
                    if !resource.labels.iter().any(|t| &LabelStringValue(t.0.clone()) == label_value) {
                        resource.labels.push(LabelStringValue(label_value.0.clone()));
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                    }
                }
            });
        },
        LabelResourceID::Folder(id) => {
            folder_uuid_to_metadata.with_mut(|folders| {
                if let Some(resource) = folders.get_mut(id) {
                    if !resource.labels.iter().any(|t| &LabelStringValue(t.0.clone()) == label_value) {
                        resource.labels.push(LabelStringValue(label_value.0.clone()));
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                    }
                }
            });
        },
        LabelResourceID::Disk(id) => {
            DISKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
        LabelResourceID::Drive(id) => {
            DRIVES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut drive) = store.get(id) {
                    if !drive.labels.iter().any(|t| t == label_value) {
                        drive.labels.push(label_value.clone());
                        store.insert(id.clone(), drive);
                    }
                }
            });
        },
        LabelResourceID::DirectoryPermission(id) => {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        LabelResourceID::SystemPermission(id) => {
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        LabelResourceID::GroupInvite(id) => {
            INVITES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
        LabelResourceID::Group(id) => {
            GROUPS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
        LabelResourceID::Webhook(id) => {
            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
        LabelResourceID::Label(id) => {
            LABELS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    if !resource.labels.iter().any(|t| t == label_value) {
                        resource.labels.push(label_value.clone());
                        store.insert(id.clone(), resource);
                    }
                }
            });
        },
    };

    // Add resource to the label's resource list
    LABELS_BY_ID_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        if let Some(mut label) = store.get(&label_id) {
            if !label.resources.iter().any(|r| r == resource_id) {
                label.resources.push(resource_id.clone());
                store.insert(label_id.clone(), label);
            }
        }
    });

    Ok(())
}

/// Remove a label from a resource
pub fn remove_label_from_resource(resource_id: &LabelResourceID, label_value: &LabelStringValue) -> Result<(), String> {
    // First, make sure the resource exists
    let resource_exists = match resource_id {
        LabelResourceID::ApiKey(id) => APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Contact(id) => CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::File(id) => file_uuid_to_metadata.contains_key(id),
        LabelResourceID::Folder(id) => folder_uuid_to_metadata.contains_key(id),
        LabelResourceID::Disk(id) => DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Drive(id) => DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::DirectoryPermission(id) => DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::SystemPermission(id) => SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::GroupInvite(id) => INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Group(id) => GROUPS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Webhook(id) => WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        LabelResourceID::Label(id) => LABELS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
    };

    if !resource_exists {
        return Err(format!("Resource {:?} not found", resource_id));
    }

    // Check if the label exists
    let label_id = LABELS_BY_VALUE_HASHTABLE.with(|store| {
        if let Some(id) = store.borrow().get(label_value) {
            Some(id.clone())
        } else {
            None
        }
    });

    let label_id = match label_id {
        Some(id) => id,
        None => return Err(format!("Label '{}' not found", label_value)),
    };

    // Remove label from the resource
    match resource_id {
        LabelResourceID::ApiKey(id) => {
            APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    store.insert(id.clone(), resource);
                }
            });
        },
        LabelResourceID::Contact(id) => {
            CONTACTS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    store.insert(id.clone(), resource);
                }
            });
        },
        LabelResourceID::File(id) => {
            file_uuid_to_metadata.with_mut(|files| {
                if let Some(resource) = files.get_mut(id) {
                    resource.labels.retain(|t| &LabelStringValue(t.0.clone()) != label_value);
                    resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                }
            });
        },
        LabelResourceID::Folder(id) => {
            folder_uuid_to_metadata.with_mut(|folders| {
                if let Some(resource) = folders.get_mut(id) {
                    resource.labels.retain(|t| &LabelStringValue(t.0.clone()) != label_value);
                    resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                }
            });
        },
        LabelResourceID::Disk(id) => {
            DISKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    store.insert(id.clone(), resource);
                }
            });
        },
        LabelResourceID::Drive(id) => {
            DRIVES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut drive) = store.get(id) {
                    if !drive.labels.iter().any(|t| t == label_value) {
                        drive.labels.push(label_value.clone());
                        store.insert(id.clone(), drive);
                    }
                }
            });
        },
        LabelResourceID::DirectoryPermission(id) => {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.labels.retain(|t| t != label_value);
                    resource.last_modified_at = ic_cdk::api::time();
                }
            });
        },
        LabelResourceID::SystemPermission(id) => {
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.labels.retain(|t| t != label_value);
                    resource.last_modified_at = ic_cdk::api::time();
                }
            });
        },
        LabelResourceID::GroupInvite(id) => {
            INVITES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    resource.last_modified_at = ic_cdk::api::time();
                    store.insert(id.clone(), resource);
                }
            });
        },
        LabelResourceID::Group(id) => {
            GROUPS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    resource.last_modified_at = ic_cdk::api::time();
                    store.insert(id.clone(), resource);
                }
            });
        },
        LabelResourceID::Webhook(id) => {
            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    store.insert(id.clone(), resource);
                }
            });
        },
        LabelResourceID::Label(id) => {
            LABELS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(mut resource) = store.get(id) {
                    resource.labels.retain(|t| t != label_value);
                    store.insert(id.clone(), resource);
                }
            });
        },
    };

    // Remove resource from the label's resource list
    LABELS_BY_ID_HASHTABLE.with(|store| {
        let store_ref = store.borrow();
        if let Some(label) = store_ref.get(&label_id) { // Using label_id without &
            // Create a modified copy
            let mut updated_label = label.clone();
            updated_label.resources.retain(|r| r != resource_id);
            
            // If this was the last resource using this label, we might want to clean up
            let is_empty = updated_label.resources.is_empty();
            let label_value = updated_label.value.clone(); // Save for later use
            
            // Insert the updated copy back
            drop(store_ref);
            store.borrow_mut().insert(label_id.clone(), updated_label.clone());
            
            // Handle cleanup if needed
            if is_empty {
                debug_log!("Label '{}' no longer has any resources", label_value);
                
                // Remove from LABELS_BY_VALUE_HASHTABLE
                LABELS_BY_VALUE_HASHTABLE.with(|v_store| {
                    v_store.borrow_mut().remove(&label_value);
                });
                
                // Remove from LABELS_BY_TIME_LIST using the pattern from reference
                LABELS_BY_TIME_LIST.with(|t_store| {
                    let mut new_vec = StableVec::init(
                        MEMORY_MANAGER.with(|m| m.borrow().get(LABELS_BY_TIME_MEMORY_ID))
                    ).expect("Failed to initialize new StableVec");
                    
                    // Copy all items except the one to be deleted
                    let t_store_ref = t_store.borrow();
                    
                    for i in 0..t_store_ref.len() {
                        if let Some(id) = t_store_ref.get(i) {
                            if id != label_id { // Compare both as references
                                new_vec.push(&id);
                            }
                        }
                    }
                    
                    // Replace the old vector with the new one
                    drop(t_store_ref);
                    *t_store.borrow_mut() = new_vec;
                });
            }
        }
    });

    Ok(())
}


pub fn update_label_string_value(
    label_id: &LabelID,
    new_value: &LabelStringValue
) -> Result<(), String> {
    // Get the label to access its resources
    let label = LABELS_BY_ID_HASHTABLE.with(|store| {
        store.borrow().get(label_id).clone()
    });
    
    let label = match label {
        Some(label) => label,
        None => return Err(format!("Label with ID {} not found", label_id)),
    };
    
    // Update all resources that have this label
    let resources = label.resources.clone();
    
    // Remove the old label from all resources
    for resource_id in &resources {
        if let Err(err) = remove_label_from_resource(resource_id, &label.value) {
            debug_log!("Error removing old label value from resource: {}", err);
            // Continue with other resources even if this one fails
        }
    }
    
    // Update the label value in the value hashtable
    LABELS_BY_VALUE_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        store.remove(&label.value);
        store.insert(new_value.clone(), label_id.clone());
    });
    
    // Add the new label to all resources
    for resource_id in &resources {
        if let Err(err) = add_label_to_resource(resource_id, new_value) {
            debug_log!("Error adding new label value to resource: {}", err);
            // Continue with other resources even if this one fails
        }
    }
    
    Ok(())
}