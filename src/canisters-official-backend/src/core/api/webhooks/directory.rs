// src/core/api/webhooks/directory.rs

use crate::{core::state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{FileID, FolderID}}, group_invites::types::GroupInvite, groups::{state::state::GROUPS_BY_ID_HASHTABLE, types::{Group, GroupID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel, WebhookIDList}}}, rest::webhooks::types::{DirectoryWebhookData, FileWebhookData, FolderWebhookData}};
use crate::rest::webhooks::types::{
    WebhookEventPayload, 
    WebhookEventData, 
    WebhookResourceData,
    GroupInviteWebhookData
};
use ic_cdk::{api::management_canister::http_request::{
    http_request, 
    HttpMethod,
    HttpHeader,
    CanisterHttpRequestArgument
}};
use ic_cdk::spawn;
use serde_json;

pub fn get_active_file_webhooks(
    file_id: &FileID, 
    event: WebhookEventLabel,
) -> Vec<Webhook> {
    let mut all_webhooks = Vec::new();
    
    // Get webhooks for the current file
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID(file_id.0.clone()))
            .map(|list| list.clone())
            .unwrap_or(WebhookIDList {
                webhooks: [].to_vec()
            })
    });

    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        all_webhooks.extend(
            webhook_ids.webhooks.into_iter()
                .filter_map(|id| store.get(&id).clone())
                .filter(|webhook| webhook.active && webhook.event == event)
        );
    });

    // Check if we should look for parent folder webhooks
    let should_check_parents = matches!(
        event,
        | WebhookEventLabel::SubfileViewed
        | WebhookEventLabel::SubfileCreated
        | WebhookEventLabel::SubfileUpdated
        | WebhookEventLabel::SubfolderDeleted
        | WebhookEventLabel::SubfolderShared
    );

    if !should_check_parents {
        return all_webhooks;
    }
    if file_id.to_string() == WebhookAltIndexID::file_created_slug().to_string() {
        return all_webhooks
    }

    // Get parent folder recursion depth
    let parent_recursion_depth = 20;
    let recursion_depth = match parent_recursion_depth {
        depth if depth > 0 => depth,
        _ => return all_webhooks,
    };

    // Start with the file's parent folder
    let mut current_folder_id = file_uuid_to_metadata
        .get(file_id)
        .and_then(|file| Some(file.parent_folder_uuid));
    let mut current_depth = 0;

    // Traverse up the parent folders
    while let Some(folder_id) = current_folder_id {
        if current_depth >= recursion_depth {
            break;
        }

        if let Some(folder_metadata) = folder_uuid_to_metadata.get(&folder_id) {
            // Stop if we hit a sovereign permissions folder
            if folder_metadata.has_sovereign_permissions {
                break;
            }

            // Get webhooks for this parent folder
            let parent_webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
                store.borrow()
                    .get(&WebhookAltIndexID(folder_id.0.clone()))
                    .map(|list| list.clone())
                    .unwrap_or(WebhookIDList {
                        webhooks: [].to_vec()
                    })
            });

            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let store = store.borrow();
                all_webhooks.extend(
                    parent_webhook_ids.webhooks.into_iter()
                        .filter_map(|id| store.get(&id).clone())
                        .filter(|webhook| webhook.active && webhook.event == event)
                );
            });

            // Move to next parent
            current_folder_id = folder_metadata.parent_folder_uuid;
            current_depth += 1;
        } else {
            break;
        }
    }

    all_webhooks
}

pub fn get_active_folder_webhooks(
    folder_id: &FolderID, 
    event: WebhookEventLabel,
) -> Vec<Webhook> {
    let mut all_webhooks = Vec::new();
    
    // Get webhooks for the current folder
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID(folder_id.0.clone()))
            .map(|list| list.clone())
            .unwrap_or(WebhookIDList {
                webhooks: [].to_vec()
            })
    });

    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        all_webhooks.extend(
            webhook_ids.webhooks.into_iter()
                .filter_map(|id| store.get(&id).clone())
                .filter(|webhook| webhook.active && webhook.event == event)
        );
    });

    // Check if we should look for parent folder webhooks
    let should_check_parents = matches!(
        event,
        | WebhookEventLabel::SubfolderViewed
        | WebhookEventLabel::SubfolderCreated
        | WebhookEventLabel::SubfolderUpdated
        | WebhookEventLabel::SubfolderDeleted
        | WebhookEventLabel::SubfolderShared
    );

    if !should_check_parents {
        return all_webhooks;
    }
    if folder_id.to_string() == WebhookAltIndexID::folder_created_slug().to_string() {
        return all_webhooks
    }

    // Get parent folder recursion depth
    let parent_recursion_depth = 20;
    let recursion_depth = match parent_recursion_depth {
        (depth) if depth > 0 => depth,
        _ => return all_webhooks,
    };

    // Start with the current folder's parent
    let mut current_folder_id = folder_uuid_to_metadata
        .get(folder_id)
        .and_then(|folder| folder.parent_folder_uuid.clone());
    let mut current_depth = 0;

    // Traverse up the parent folders
    while let Some(parent_id) = current_folder_id {
        if current_depth >= recursion_depth {
            break;
        }

        if let Some(folder_metadata) = folder_uuid_to_metadata.get(&parent_id) {
            // Stop if we hit a sovereign permissions folder
            if folder_metadata.has_sovereign_permissions {
                break;
            }

            // Get webhooks for this parent folder
            let parent_webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
                store.borrow()
                    .get(&WebhookAltIndexID(parent_id.0.clone()))
                    .map(|list| list.clone())
                    .unwrap_or(WebhookIDList {
                        webhooks: [].to_vec()
                    })
            });

            WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
                let store = store.borrow();
                all_webhooks.extend(
                    parent_webhook_ids.webhooks
                        .into_iter()
                        .filter_map(|id| store.get(&id).clone())
                        .filter(|webhook| webhook.active && webhook.event == event)
                );
            });

            // Move to next parent
            current_folder_id = folder_metadata.parent_folder_uuid;
            current_depth += 1;
        } else {
            break;
        }
    }

    all_webhooks
}



pub fn fire_directory_webhook(
    event: WebhookEventLabel,
    webhooks: Vec<Webhook>,
    before_snap: Option<DirectoryWebhookData>,
    after_snap: Option<DirectoryWebhookData>,
    notes: Option<String>,
) {
    let timestamp_ms = ic_cdk::api::time() / 1_000_000;
    for webhook in webhooks {
        let payload = WebhookEventPayload {
            event: event.to_string(),
            timestamp_ms,
            nonce: timestamp_ms,
            notes: notes.clone(),
            webhook_id: webhook.id.clone(),
            webhook_alt_index: webhook.alt_index.clone(),
            payload: WebhookEventData {
                before: before_snap.clone().map(|snap| match snap {
                    DirectoryWebhookData::File(data) => WebhookResourceData::File(data),
                    DirectoryWebhookData::Folder(data) => WebhookResourceData::Folder(data),
                    DirectoryWebhookData::Subfile(data) => WebhookResourceData::Subfile(data),
                    DirectoryWebhookData::Subfolder(data) => WebhookResourceData::Subfolder(data),
                    DirectoryWebhookData::ShareTracking(data) => WebhookResourceData::ShareTracking(data),
                }),
                after: after_snap.clone().map(|snap| match snap {
                    DirectoryWebhookData::File(data) => WebhookResourceData::File(data),
                    DirectoryWebhookData::Folder(data) => WebhookResourceData::Folder(data),
                    DirectoryWebhookData::Subfile(data) => WebhookResourceData::Subfile(data),
                    DirectoryWebhookData::Subfolder(data) => WebhookResourceData::Subfolder(data),
                    DirectoryWebhookData::ShareTracking(data) => WebhookResourceData::ShareTracking(data),
                }),
            },
        };

        // Serialize payload for this webhook
        if let Ok(body) = serde_json::to_vec(&payload) {
            let request = CanisterHttpRequestArgument {
                url: webhook.url.clone(),
                method: HttpMethod::POST,
                headers: vec![
                    HttpHeader {
                        name: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    },
                    HttpHeader {
                        name: "signature".to_string(),
                        value: webhook.signature.clone(),
                    },
                ],
                body: Some(body),
                max_response_bytes: Some(0),
                transform: None,
            };

            spawn(async move {
                let cycles: u128 = 1_000_000_000;
                let _ = http_request(request, cycles).await;
            });
        }
    }
}