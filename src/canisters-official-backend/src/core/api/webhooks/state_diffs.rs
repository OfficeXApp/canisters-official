// src/core/api/webhooks/diffs.rs

use crate::{core::{api::{replay::diff::{apply_entire_state, apply_state_diff, update_checksum_for_state_diff}, uuid::generate_uuidv4}, state::{drives::{state::state::{DRIVE_ID, DRIVE_STATE_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, URL_ENDPOINT}, types::{DriveStateDiffID, DriveStateDiffImplementationType, DriveStateDiffString, StateChecksum, StateDiffRecord}}, group_invites::types::GroupInvite, groups::{state::state::GROUPS_BY_ID_HASHTABLE, types::{Group, GroupID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel}}}, types::IDPrefix}, rest::webhooks::types::DriveStateDiffWebhookData};
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

pub fn get_active_state_diff_webhooks() -> Vec<Webhook> {
    let webhook_ids = WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        store.borrow()
            .get(&WebhookAltIndexID(WebhookAltIndexID::state_diffs_slug().to_string()))
            .cloned()
            .unwrap_or_default()
    });

    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        let store = store.borrow();
        webhook_ids.into_iter()
            .filter_map(|id| store.get(&id).cloned())
            .filter(|webhook| webhook.active && webhook.event == WebhookEventLabel::DriveStateDiffs)
            .collect()
    })
}

pub fn fire_state_diff_webhooks(
    forward_diff: DriveStateDiffString,
    backward_diff: DriveStateDiffString,
    forward_checksum: StateChecksum,
    backward_checksum: StateChecksum,
    notes: Option<String>
) {
    let drive_state_diff_id = DriveStateDiffID(generate_uuidv4(IDPrefix::DriveStateDiffID));
    // we skip mark_claimed_uuid as that will be responsibility of the webhook, and we generaete this drive_state_diff_id on the fly anyways
    let timestamp_ns = DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get());
    let webhooks = get_active_state_diff_webhooks();
    
    for webhook in webhooks {
        let payload = WebhookEventPayload {
            event: WebhookEventLabel::DriveStateDiffs.to_string(),
            timestamp_ms: timestamp_ns / 1_000_000, // Convert to milliseconds
            nonce: timestamp_ns.clone(),
            notes: notes.clone(),
            webhook_id: webhook.id.clone(),
            webhook_alt_index: webhook.alt_index.clone(),
            payload: WebhookEventData {
                before: None,
                after: Some(WebhookResourceData::StateDiffs(DriveStateDiffWebhookData{ 
                    data: StateDiffRecord {
                        id: drive_state_diff_id.clone(),
                        timestamp_ns: timestamp_ns,
                        implementation: DriveStateDiffImplementationType::RustIcpCanister,
                        diff_forward: forward_diff.clone(),
                        diff_backward: backward_diff.clone(),
                        notes: notes.clone(),
                        drive_id: DRIVE_ID.with(|id| id.clone()),
                        endpoint_url: URL_ENDPOINT.with(|url| url.borrow().clone()),
                        checksum_forward: forward_checksum.clone(),
                        checksum_backward: backward_checksum.clone(),
                    }
                }))
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