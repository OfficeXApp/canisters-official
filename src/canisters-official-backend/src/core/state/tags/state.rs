// src/core/state/tags/state.rs

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{
    core::{
        api::types::DirectoryIDError,
        state::{
            api_keys::types::ApiKeyID,
            api_keys::state::state::APIKEYS_BY_ID_HASHTABLE,
            contacts::types::Contact,
            contacts::state::state::CONTACTS_BY_ID_HASHTABLE,
            directory::types::{FileUUID, FolderUUID},
            directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata},
            disks::types::DiskID,
            disks::state::state::DISKS_BY_ID_HASHTABLE,
            drives::types::DriveID,
            drives::state::state::DRIVES_BY_ID_HASHTABLE,
            permissions::types::{DirectoryPermissionID, SystemPermissionID},
            permissions::state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE},
            team_invites::types::TeamInviteID,
            team_invites::state::state::INVITES_BY_ID_HASHTABLE,
            teams::types::TeamID,
            teams::state::state::TEAMS_BY_ID_HASHTABLE,
            webhooks::types::WebhookID,
            webhooks::state::state::WEBHOOKS_BY_ID_HASHTABLE,
            tags::types::{TagStringValue, TagResourceID}
        },
        types::{IDPrefix, UserID}
    },
    debug_log
};

thread_local! {
    // Map tags to resources
    pub(crate) static TAGS_BY_VALUE_HASHTABLE: RefCell<HashMap<TagStringValue, Vec<TagResourceID>>> = RefCell::new(HashMap::new());
}

/// Validates a tag string to ensure it meets requirements
pub fn validate_tag(tag: &str) -> Result<TagStringValue, String> {
    // Check length
    if tag.is_empty() {
        return Err("Tag cannot be empty".to_string());
    }
    if tag.len() > 64 {
        return Err("Tag cannot exceed 64 characters".to_string());
    }

    // Check characters
    if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Tag can only contain alphanumeric characters and underscores".to_string());
    }

    // Convert to lowercase for consistency
    Ok(TagStringValue(tag.to_lowercase()))
}

/// Parse a resource ID string into the appropriate TagResourceID enum
pub fn parse_tag_resource_id(id_str: &str) -> Result<TagResourceID, DirectoryIDError> {
    // Check if the string contains a valid prefix
    if let Some(prefix_str) = id_str.splitn(2, '_').next() {
        match prefix_str {
            "ApiKeyID" => Ok(TagResourceID::ApiKey(ApiKeyID(id_str.to_string()))),
            "UserID" => Ok(TagResourceID::Contact(UserID(id_str.to_string()))),
            "FileID" => Ok(TagResourceID::File(FileUUID(id_str.to_string()))),
            "FolderID" => Ok(TagResourceID::Folder(FolderUUID(id_str.to_string()))),
            "DiskID" => Ok(TagResourceID::Disk(DiskID(id_str.to_string()))),
            "DriveID" => Ok(TagResourceID::Drive(DriveID(id_str.to_string()))),
            "DirectoryPermissionID" => Ok(TagResourceID::DirectoryPermission(DirectoryPermissionID(id_str.to_string()))),
            "SystemPermissionID" => Ok(TagResourceID::SystemPermission(SystemPermissionID(id_str.to_string()))),
            "InviteID" => Ok(TagResourceID::TeamInvite(TeamInviteID(id_str.to_string()))),
            "TeamID" => Ok(TagResourceID::Team(TeamID(id_str.to_string()))),
            "WebhookID" => Ok(TagResourceID::Webhook(WebhookID(id_str.to_string()))),
            _ => Err(DirectoryIDError::InvalidPrefix),
        }
    } else {
        Err(DirectoryIDError::MalformedID)
    }
}

/// Add a tag to a resource
pub fn add_tag_to_resource(resource_id: &TagResourceID, tag: &TagStringValue) -> Result<(), String> {
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
    };

    if !resource_exists {
        return Err(format!("Resource {:?} not found", resource_id));
    }

    // Add tag to the resource
    match resource_id {
        TagResourceID::ApiKey(id) => {
            APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    // Add tags field if not already present
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                    }
                }
            });
        },
        TagResourceID::Contact(id) => {
            CONTACTS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                    }
                }
            });
        },
        TagResourceID::File(id) => {
            file_uuid_to_metadata.with_mut(|files| {
                if let Some(resource) = files.get_mut(id) {
                    if !resource.tags.iter().any(|t| &TagStringValue(t.0.clone()) == tag) {
                        resource.tags.push(TagStringValue(tag.0.clone()));
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                    }
                }
            });
        },
        TagResourceID::Folder(id) => {
            folder_uuid_to_metadata.with_mut(|folders| {
                if let Some(resource) = folders.get_mut(id) {
                    if !resource.tags.iter().any(|t| &TagStringValue(t.0.clone()) == tag) {
                        resource.tags.push(TagStringValue(tag.0.clone()));
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                    }
                }
            });
        },
        TagResourceID::Disk(id) => {
            DISKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                    }
                }
            });
        },
        TagResourceID::Drive(id) => {
            DRIVES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                    }
                }
            });
        },
        TagResourceID::DirectoryPermission(id) => {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::SystemPermission(id) => {
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::TeamInvite(id) => {
            INVITES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::Team(id) => {
            TEAMS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                        resource.last_modified_at = ic_cdk::api::time();
                    }
                }
            });
        },
        TagResourceID::Webhook(id) => {
            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    if !resource.tags.iter().any(|t| t == tag) {
                        resource.tags.push(tag.clone());
                    }
                }
            });
        },
    };

    // Add resource to the tag's resource list
    TAGS_BY_VALUE_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        let resources = store.entry(tag.clone()).or_insert_with(Vec::new);
        
        // Check if the resource is already in the list
        if !resources.iter().any(|r| r == resource_id) {
            resources.push(resource_id.clone());
        }
    });

    Ok(())
}

/// Remove a tag from a resource
pub fn remove_tag_from_resource(resource_id: &TagResourceID, tag: &TagStringValue) -> Result<(), String> {
    // Remove tag from the resource
    let tag_found = match resource_id {
        TagResourceID::ApiKey(id) => {
            APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    len_before > resource.tags.len()
                } else {
                    false
                }
            })
        },
        TagResourceID::Contact(id) => {
            CONTACTS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    len_before > resource.tags.len()
                } else {
                    false
                }
            })
        },
        TagResourceID::File(id) => {
            file_uuid_to_metadata.with_mut(|files| {
                if let Some(resource) = files.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| &TagStringValue(t.0.clone()) != tag);
                    if len_before > resource.tags.len() {
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        },
        TagResourceID::Folder(id) => {
            folder_uuid_to_metadata.with_mut(|folders| {
                if let Some(resource) = folders.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| &TagStringValue(t.0.clone()) != tag);
                    if len_before > resource.tags.len() {
                        resource.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        },
        TagResourceID::Disk(id) => {
            DISKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    len_before > resource.tags.len()
                } else {
                    false
                }
            })
        },
        TagResourceID::Drive(id) => {
            DRIVES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    len_before > resource.tags.len()
                } else {
                    false
                }
            })
        },
        TagResourceID::DirectoryPermission(id) => {
            DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    if len_before > resource.tags.len() {
                        resource.last_modified_at = ic_cdk::api::time();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        },
        TagResourceID::SystemPermission(id) => {
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    if len_before > resource.tags.len() {
                        resource.last_modified_at = ic_cdk::api::time();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        },
        TagResourceID::TeamInvite(id) => {
            INVITES_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    if len_before > resource.tags.len() {
                        resource.last_modified_at = ic_cdk::api::time();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        },
        TagResourceID::Team(id) => {
            TEAMS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    if len_before > resource.tags.len() {
                        resource.last_modified_at = ic_cdk::api::time();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        },
        TagResourceID::Webhook(id) => {
            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(resource) = store.get_mut(id) {
                    let len_before = resource.tags.len();
                    resource.tags.retain(|t| t != tag);
                    len_before > resource.tags.len()
                } else {
                    false
                }
            })
        },
    };

    if !tag_found {
        return Err(format!("Tag '{}' not found on resource {:?}", tag.0, resource_id));
    }

    // Remove resource from the tag's resource list
    TAGS_BY_VALUE_HASHTABLE.with(|store| {
        let mut store = store.borrow_mut();
        if let Some(resources) = store.get_mut(tag) {
            resources.retain(|r| r != resource_id);
            
            // If no resources left for this tag, remove the tag entry
            if resources.is_empty() {
                store.remove(tag);
            }
        }
    });

    Ok(())
}
