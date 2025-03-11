// src/core/state/tags/state.rs

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{
    core::{
        api::{types::DirectoryIDError, uuid::generate_uuidv4},
        state::{
            api_keys::{state::state::APIKEYS_BY_ID_HASHTABLE, types::ApiKeyID}, contacts::{state::state::CONTACTS_BY_ID_HASHTABLE, types::Contact}, directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{FileID, FolderID}}, disks::{state::state::DISKS_BY_ID_HASHTABLE, types::DiskID}, drives::{state::state::DRIVES_BY_ID_HASHTABLE, types::DriveID}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE}, types::{DirectoryPermissionID, SystemPermissionID}}, tags::types::{TagResourceID, TagStringValue}, team_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::TeamInviteID}, teams::{state::state::TEAMS_BY_ID_HASHTABLE, types::TeamID}, webhooks::{state::state::WEBHOOKS_BY_ID_HASHTABLE, types::WebhookID}
        },
        types::{IDPrefix, UserID}
    },
    debug_log, rest::types::ValidationError
};

use super::types::{HexColorString, Tag, TagID};

thread_local! {
    // Map tags to resources
    pub(crate) static TAGS_BY_ID_HASHTABLE: RefCell<HashMap<TagID, Tag>> = RefCell::new(HashMap::new());
    pub(crate) static TAGS_BY_VALUE_HASHTABLE: RefCell<HashMap<TagStringValue, TagID>> = RefCell::new(HashMap::new());
    pub(crate) static TAGS_BY_TIME_LIST: RefCell<Vec<TagID>> = RefCell::new(Vec::new());
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
        if claimed.borrow().contains_key(uuid_str) {
            Err(ValidationError {
                field: "uuid".to_string(),
                message: "UUID has already been claimed".to_string(),
            })
        } else {
            Ok(())
        }
    })
}


/// Validates a tag string to ensure it meets requirements
pub fn validate_tag_value(tag_value: &str) -> Result<TagStringValue, String> {
    // Check length
    if tag_value.is_empty() {
        return Err("Tag cannot be empty".to_string());
    }
    if tag_value.len() > 64 {
        return Err("Tag cannot exceed 64 characters".to_string());
    }

    // Check characters
    if !tag_value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Tag can only contain alphanumeric characters and underscores".to_string());
    }

    // Convert to lowercase for consistency
    Ok(TagStringValue(tag_value.to_lowercase()))
}

pub fn validate_color(color: &str) -> Result<HexColorString, String> {
    // Check length
    if color.is_empty() {
        return Err("Color cannot be empty".to_string());
    }
    if color.len() != 7 {
        return Err("Color must be a 7-character hex string".to_string());
    }

    // Check characters
    if !color.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Color must be a 7-character hex string".to_string());
    }

    // Check prefix
    if !color.starts_with('#') {
        return Err("Color must start with '#'".to_string());
    }

    Ok(HexColorString(color.to_uppercase()))
}

/// Parse a resource ID string into the appropriate TagResourceID enum
pub fn parse_tag_resource_id(id_str: &str) -> Result<TagResourceID, DirectoryIDError> {
    // Check if the string contains a valid prefix
    if let Some(prefix_str) = id_str.splitn(2, '_').next() {
        match prefix_str {
            "ApiKeyID" => Ok(TagResourceID::ApiKey(ApiKeyID(id_str.to_string()))),
            "UserID" => Ok(TagResourceID::Contact(UserID(id_str.to_string()))),
            "FileID" => Ok(TagResourceID::File(FileID(id_str.to_string()))),
            "FolderID" => Ok(TagResourceID::Folder(FolderID(id_str.to_string()))),
            "DiskID" => Ok(TagResourceID::Disk(DiskID(id_str.to_string()))),
            "DriveID" => Ok(TagResourceID::Drive(DriveID(id_str.to_string()))),
            "DirectoryPermissionID" => Ok(TagResourceID::DirectoryPermission(DirectoryPermissionID(id_str.to_string()))),
            "SystemPermissionID" => Ok(TagResourceID::SystemPermission(SystemPermissionID(id_str.to_string()))),
            "InviteID" => Ok(TagResourceID::TeamInvite(TeamInviteID(id_str.to_string()))),
            "TeamID" => Ok(TagResourceID::Team(TeamID(id_str.to_string()))),
            "WebhookID" => Ok(TagResourceID::Webhook(WebhookID(id_str.to_string()))),
            "TagID" => Ok(TagResourceID::Tag(TagID(id_str.to_string()))),
            _ => Err(DirectoryIDError::InvalidPrefix),
        }
    } else {
        Err(DirectoryIDError::MalformedID)
    }
}

/// Add a tag to a resource
pub fn add_tag_to_resource(resource_id: &TagResourceID, tag_value: &TagStringValue) -> Result<(), String> {
    // First, make sure the resource exists
    let resource_exists = match resource_id {
        TagResourceID::ApiKey(id) => APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Contact(id) => CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::File(id) => file_uuid_to_metadata.contains_key(id),
        TagResourceID::Folder(id) => folder_uuid_to_metadata.contains_key(id),
        TagResourceID::Disk(id) => DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Drive(id) => DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::DirectoryPermission(id) => DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::SystemPermission(id) => SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::TeamInvite(id) => INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Team(id) => TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Webhook(id) => WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Tag(id) => TAGS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
    };

    if !resource_exists {
        return Err(format!("Resource {:?} not found", resource_id));
    }

    // Check if the tag exists, create it if not
    let tag_id = TAGS_BY_VALUE_HASHTABLE.with(|store| {
        // Clone the TagID if found to avoid lifetime issues
        if let Some(id) = store.borrow().get(tag_value) {
            Some(id.clone())
        } else {
            None
        }
    }).unwrap_or_else(|| {
        let tag_id = TagID(generate_uuidv4(IDPrefix::TagID));
        let tag = Tag {
            id: tag_id.clone(),
            value: tag_value.clone(),
            public_note: None,
            private_note: None,
            color: HexColorString("#FFFFFF".to_string()),
            created_at: ic_cdk::api::time() / 1_000_000,
            last_updated_at: ic_cdk::api::time() / 1_000_000,
            resources: vec![resource_id.clone()],
            tags: vec![],
            created_by: UserID("".to_string()),
            external_id: None,
            external_payload: None,
        };
    
        TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(tag_id.clone(), tag);
        });
        TAGS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().insert(tag_value.clone(), tag_id.clone());
        });
        TAGS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(tag_id.clone());
        });
    
        tag_id
    });

    // Add tag to the resource
    match resource_id {
        TagResourceID::ApiKey(id) => {
            APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    // Add tags field if not already present
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                    }
                }
            });
        },
        TagResourceID::Contact(id) => {
            CONTACTS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                    }
                }
            });
        },
        TagResourceID::File(id) => {
            file_uuid_to_metadata.with_mut(|files| {
                if let Some(resource) = files.get_mut(id) {
                    if !resource.tags.iter().any(|t| &TagStringValue(t.0.clone()) == tag_value) {
                        resource.tags.push(TagStringValue(tag_value.0.clone()));
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                    }
                }
            });
        },
        TagResourceID::Folder(id) => {
            folder_uuid_to_metadata.with_mut(|folders| {
                if let Some(resource) = folders.get_mut(id) {
                    if !resource.tags.iter().any(|t| &TagStringValue(t.0.clone()) == tag_value) {
                        resource.tags.push(TagStringValue(tag_value.0.clone()));
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                    }
                }
            });
        },
        TagResourceID::Disk(id) => {
            DISKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                    }
                }
            });
        },
        TagResourceID::Drive(id) => {
            DRIVES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                    }
                }
            });
        },
        TagResourceID::DirectoryPermission(id) => {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::SystemPermission(id) => {
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::TeamInvite(id) => {
            INVITES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::Team(id) => {
            TEAMS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::Webhook(id) => {
            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                    }
                }
            });
        },
        TagResourceID::Tag(id) => {
            TAGS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag_value) {
                        resource.tags.push(tag_value.clone());
                    }
                }
            });
        },
    };

    // Add resource to the tag's resource list
    TAGS_BY_ID_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        if let Some(tag) = store.get_mut(&tag_id) {
            if !tag.resources.iter().any(|r| r == resource_id) {
                tag.resources.push(resource_id.clone());
            }
        }
    });

    Ok(())
}

/// Remove a tag from a resource
pub fn remove_tag_from_resource(resource_id: &TagResourceID, tag_value: &TagStringValue) -> Result<(), String> {
    // First, make sure the resource exists
    let resource_exists = match resource_id {
        TagResourceID::ApiKey(id) => APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Contact(id) => CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::File(id) => file_uuid_to_metadata.contains_key(id),
        TagResourceID::Folder(id) => folder_uuid_to_metadata.contains_key(id),
        TagResourceID::Disk(id) => DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Drive(id) => DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::DirectoryPermission(id) => DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::SystemPermission(id) => SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::TeamInvite(id) => INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Team(id) => TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Webhook(id) => WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
        TagResourceID::Tag(id) => TAGS_BY_ID_HASHTABLE.with(|store| store.borrow().contains_key(id)),
    };

    if !resource_exists {
        return Err(format!("Resource {:?} not found", resource_id));
    }

    // Check if the tag exists
    let tag_id = TAGS_BY_VALUE_HASHTABLE.with(|store| {
        if let Some(id) = store.borrow().get(tag_value) {
            Some(id.clone())
        } else {
            None
        }
    });

    let tag_id = match tag_id {
        Some(id) => id,
        None => return Err(format!("Tag '{}' not found", tag_value)),
    };

    // Remove tag from the resource
    match resource_id {
        TagResourceID::ApiKey(id) => {
            APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                }
            });
        },
        TagResourceID::Contact(id) => {
            CONTACTS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                }
            });
        },
        TagResourceID::File(id) => {
            file_uuid_to_metadata.with_mut(|files| {
                if let Some(resource) = files.get_mut(id) {
                    resource.tags.retain(|t| &TagStringValue(t.0.clone()) != tag_value);
                    resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                }
            });
        },
        TagResourceID::Folder(id) => {
            folder_uuid_to_metadata.with_mut(|folders| {
                if let Some(resource) = folders.get_mut(id) {
                    resource.tags.retain(|t| &TagStringValue(t.0.clone()) != tag_value);
                    resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                }
            });
        },
        TagResourceID::Disk(id) => {
            DISKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                }
            });
        },
        TagResourceID::Drive(id) => {
            DRIVES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                }
            });
        },
        TagResourceID::DirectoryPermission(id) => {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                    resource.last_modified_at = ic_cdk::api::time();
                }
            });
        },
        TagResourceID::SystemPermission(id) => {
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                    resource.last_modified_at = ic_cdk::api::time();
                }
            });
        },
        TagResourceID::TeamInvite(id) => {
            INVITES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                    resource.last_modified_at = ic_cdk::api::time();
                }
            });
        },
        TagResourceID::Team(id) => {
            TEAMS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                    resource.last_modified_at = ic_cdk::api::time();
                }
            });
        },
        TagResourceID::Webhook(id) => {
            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                }
            });
        },
        TagResourceID::Tag(id) => {
            TAGS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    resource.tags.retain(|t| t != tag_value);
                }
            });
        },
    };

    // Remove resource from the tag's resource list
    TAGS_BY_ID_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        if let Some(tag) = store.get_mut(&tag_id) {
            tag.resources.retain(|r| r != resource_id);
            
            // If this was the last resource using this tag, we might want to clean up
            // This is optional - you may prefer to keep empty tags for future use
            if tag.resources.is_empty() {
                debug_log!("Tag '{}' no longer has any resources", tag_value);
                // Uncomment to delete empty tags
                TAGS_BY_VALUE_HASHTABLE.with(|v_store| {
                    v_store.borrow_mut().remove(&tag.value);
                });
                TAGS_BY_TIME_LIST.with(|t_store| {
                    let mut t_store = t_store.borrow_mut();
                    if let Some(pos) = t_store.iter().position(|t| t == &tag_id) {
                        t_store.remove(pos);
                    }
                });
            }
        }
    });

    Ok(())
}


pub fn update_tag_string_value(
    tag_id: &TagID,
    new_value: &TagStringValue
) -> Result<(), String> {
    // Get the tag to access its resources
    let tag = TAGS_BY_ID_HASHTABLE.with(|store| {
        store.borrow().get(tag_id).cloned()
    });
    
    let tag = match tag {
        Some(tag) => tag,
        None => return Err(format!("Tag with ID {} not found", tag_id)),
    };
    
    // Update all resources that have this tag
    let resources = tag.resources.clone();
    
    // Remove the old tag from all resources
    for resource_id in &resources {
        if let Err(err) = remove_tag_from_resource(resource_id, &tag.value) {
            debug_log!("Error removing old tag value from resource: {}", err);
            // Continue with other resources even if this one fails
        }
    }
    
    // Update the tag value in the value hashtable
    TAGS_BY_VALUE_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        store.remove(&tag.value);
        store.insert(new_value.clone(), tag_id.clone());
    });
    
    // Add the new tag to all resources
    for resource_id in &resources {
        if let Err(err) = add_tag_to_resource(resource_id, new_value) {
            debug_log!("Error adding new tag value to resource: {}", err);
            // Continue with other resources even if this one fails
        }
    }
    
    Ok(())
}