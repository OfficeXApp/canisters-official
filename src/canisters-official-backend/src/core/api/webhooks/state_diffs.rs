// src/core/api/webhooks/diffs.rs

use crate::{core::{api::uuid::{generate_unique_id, update_checksum_for_state_diff}, state::{drives::{state::state::{DRIVE_ID, DRIVE_STATE_DIFF_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, URL_ENDPOINT}, types::{DriveStateDiffID, DriveStateDiffImplementationType, DriveStateDiffRecord, DriveStateDiffString}}, team_invites::types::Team_Invite, teams::{state::state::TEAMS_BY_ID_HASHTABLE, types::{Team, TeamID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel}}}, types::IDPrefix}, rest::webhooks::types::DriveStateDiffWebhookData};
use crate::rest::webhooks::types::{
    WebhookEventPayload, 
    WebhookEventData, 
    WebhookResourceData,
    TeamInviteWebhookData
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
    diff: DriveStateDiffString,
    notes: Option<String>
) {
    let drive_state_diff_id = DriveStateDiffID(generate_unique_id(IDPrefix::DriveStateDiffID, ""));
    let timestamp_ns = DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get());
    let webhooks = get_active_state_diff_webhooks();

    update_checksum_for_state_diff(diff.clone());

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
                    data: DriveStateDiffRecord {
                        id: drive_state_diff_id.clone(),
                        timestamp_ns: timestamp_ns,
                        implementation: DriveStateDiffImplementationType::RustIcpCanister,
                        diff: diff.clone(),
                        notes: notes.clone(),
                        drive_id: DRIVE_ID.with(|id| id.clone()),
                        url_endpoint: URL_ENDPOINT.with(|url| url.borrow().clone()),
                        checksum: DRIVE_STATE_DIFF_CHECKSUM.with(|checksum| checksum.borrow().clone()),
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