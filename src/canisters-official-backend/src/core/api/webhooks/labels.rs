// src/core/api/webhooks/labels.rs

use crate::core::state::{labels::types::{Label, LabelID}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel, WebhookIDList}}};
use crate::rest::webhooks::types::{
    WebhookEventPayload, 
    WebhookEventData, 
    WebhookResourceData,
    LabelWebhookData
};
use ic_cdk::{api::management_canister::http_request::{
    http_request, 
    HttpMethod,
    HttpHeader,
    CanisterHttpRequestArgument
}};
use ic_cdk::spawn;
use serde_json;

pub fn get_active_label_webhooks(label_id: &LabelID, event: WebhookEventLabel) -> Vec<Webhook> {
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID(label_id.0.clone()))
            .map(|list| list.clone())
            .unwrap_or_else(|| WebhookIDList { webhooks: Vec::new() }) 
    });

    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        webhook_ids.iter()  
            .filter_map(|id| store.get(id).clone())
            .filter(|webhook| webhook.active && webhook.event == event)
            .collect()
    })
}

pub fn fire_label_webhook(
    event: WebhookEventLabel,
    webhooks: Vec<Webhook>,
    before_snap: Option<LabelWebhookData>,
    after_snap: Option<LabelWebhookData>,
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
                before: before_snap.clone().map(|snap| WebhookResourceData::Label(snap)),
                after: after_snap.clone().map(|snap| WebhookResourceData::Label(snap)),
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