// src/core/api/webhooks/group_invites.rs

use crate::core::state::{group_invites::types::GroupInvite, groups::{state::state::GROUPS_BY_ID_HASHTABLE, types::{Group, GroupID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel, WebhookIDList}}};
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

pub fn get_active_group_invite_webhooks(group_id: &GroupID, event: WebhookEventLabel) -> Vec<Webhook> {
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID(group_id.0.clone()))
            .map(|list| list.clone())
            .unwrap_or(WebhookIDList { webhooks: [].to_vec() })
    });

    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        webhook_ids.webhooks
            .into_iter()
            .filter_map(|id| store.get(&id).clone())
            .filter(|webhook| webhook.active && webhook.event == event)
            .collect()
    })
}

pub fn fire_group_invite_webhook(
    event: WebhookEventLabel,
    webhooks: Vec<Webhook>,
    before_snap: Option<GroupInviteWebhookData>,
    after_snap: Option<GroupInviteWebhookData>,
    notes: Option<String>
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
                before: before_snap.clone().map(|snap| WebhookResourceData::GroupInvite(snap)),
                after: after_snap.clone().map(|snap| WebhookResourceData::GroupInvite(snap)),
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