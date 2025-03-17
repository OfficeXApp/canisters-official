// src/core/api/webhooks/group_invites.rs

use crate::{core::{
    state::{group_invites::types::GroupInvite, groups::{state::state::GROUPS_BY_ID_HASHTABLE, types::{Group, GroupID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel}}},
    types::UserID,
}, debug_log, rest::organization::types::InboxOrgRequestBody};
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

pub fn get_superswap_user_webhooks(event: WebhookEventLabel) -> Vec<Webhook> {
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID::superswap_user_slug())
            .cloned()
            .unwrap_or_default()
    });

    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        webhook_ids.into_iter()
            .filter_map(|id| store.get(&id).cloned())
            .filter(|webhook| webhook.active && webhook.event == event)
            .collect()
    })
}


pub fn get_org_inbox_webhooks(topic: Option<&String>) -> Vec<Webhook> {
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID::inbox_new_notif_slug())
            .cloned()
            .unwrap_or_default()
    });


    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        webhook_ids.into_iter()
            .filter_map(|id| store.get(&id).cloned())
            .filter(|webhook| {
                
                if !webhook.active || webhook.event != WebhookEventLabel::OrganizationInboxNewNotif {
                    return false;
                }
                
                match (topic, webhook.filters.as_str()) {
                    // No topic in request but webhook has filter - no match
                    (None, filters) if !filters.is_empty() => {
                        false
                    },
                    
                    // Topic in request but no filter in webhook - no match
                    (Some(_), filters) if filters.is_empty() => {
                        false
                    },
                    
                    // Both topic and filter exist - try to match
                    (Some(request_topic), filters) if !filters.is_empty() => {
                        
                        match serde_json::from_str::<serde_json::Value>(filters) {
                            Ok(filter_json) => {
                                match filter_json.get("topic") {
                                    Some(filter_topic) => {
                                        match filter_topic.as_str() {
                                            Some(filter_topic_str) => {
                                                let matches = filter_topic_str == request_topic;
                                                matches
                                            },
                                            None => {
                                                debug_log!("Filter topic is not a string");
                                                false
                                            }
                                        }
                                    },
                                    None => {
                                        debug_log!("No 'topic' key in filter JSON");
                                        false
                                    }
                                }
                            },
                            Err(e) => {
                                debug_log!("Failed to parse filter JSON: {}", e);
                                false
                            }
                        }
                    },
                    
                    // No topic and no filter - catch-all case
                    _ => {
                        debug_log!("Catch-all case - no topic and no filter");
                        true
                    }
                }
            })
            .collect()
    })
}

pub fn fire_superswap_user_webhook(
    event: WebhookEventLabel,
    webhooks: Vec<Webhook>,
    before_snap: Option<UserID>,
    after_snap: Option<UserID>,
    notes: Option<String>
) {
    let timestamp_ms = ic_cdk::api::time() / 1_000_000;
    
    // First issue: WebhookResourceData::None doesn't exist in the enum
    // Second issue: The before option handling is incorrect
    let before = before_snap.map(|user_id| {
        Some(WebhookResourceData::SuperswapUserID(user_id))
    });
    
    // Same issue with after_snap
    let after = after_snap.map(|user_id| {
        Some(WebhookResourceData::SuperswapUserID(user_id))
    });
    
    for webhook in webhooks {
        let payload = WebhookEventPayload {
            event: event.to_string(),
            timestamp_ms,
            nonce: timestamp_ms,
            notes: notes.clone(),
            webhook_id: webhook.id.clone(),
            webhook_alt_index: webhook.alt_index.clone(),
            payload: WebhookEventData {
                // These need to be Options of WebhookResourceData, not UserID
                before: before.clone().flatten(),
                after: after.clone().flatten(),
            },
        };
        
        // The rest of the function looks correct
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


pub async fn fire_org_inbox_new_notif_webhook(
    event: WebhookEventLabel,
    webhooks: Vec<Webhook>,
    before_snap: Option<InboxOrgRequestBody>,
    after_snap: Option<InboxOrgRequestBody>,
    notes: Option<String>
) {
    let timestamp_ms = ic_cdk::api::time() / 1_000_000;

    let before_snap_payload = match before_snap {
        Some(before) => Some(WebhookResourceData::OrgInboxNewNotif(before.clone())),
        None => None,
    };

    let after_snap_payload = match after_snap {
        Some(after) => Some(WebhookResourceData::OrgInboxNewNotif(after.clone())),
        None => None,
    };
    
    // Process webhooks sequentially to ensure they all complete
    for webhook in webhooks {
        let payload = WebhookEventPayload {
            event: event.to_string(),
            timestamp_ms,
            nonce: timestamp_ms,
            notes: notes.clone(),
            webhook_id: webhook.id.clone(),
            webhook_alt_index: webhook.alt_index.clone(),
            payload: WebhookEventData {
                before: before_snap_payload.clone(),
                after: after_snap_payload.clone()
            },
        };
        
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

            // Execute the HTTP request and await its completion
            let cycles: u128 = 1_000_000_000;
            let _ = http_request(request, cycles).await;
        }
    }
}