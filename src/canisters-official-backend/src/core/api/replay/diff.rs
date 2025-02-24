// src/core/api/replay/diff.rs

use serde_diff::{Diff, SerdeDiff};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::core::{api::webhooks::state_diffs::{fire_state_diff_webhooks, get_active_state_diff_webhooks}, state::{api_keys::{state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue}}, drives::types::DriveStateDiffString}, types::UserID};

// Define a type to represent the entire state
#[derive(SerdeDiff, Serialize, Deserialize, Clone)]
pub struct EntireState {
    APIKEYS_BY_VALUE_HASHTABLE: HashMap<ApiKeyValue, ApiKeyID>,
    APIKEYS_BY_ID_HASHTABLE: HashMap<ApiKeyID, ApiKey>,
    USERS_APIKEYS_HASHTABLE: HashMap<UserID, Vec<ApiKeyID>>
    // Add more state hashtables as needed
}

fn snapshot_entire_state() -> EntireState {
    EntireState {
        // Clone each hashtable from your thread_local storage
        APIKEYS_BY_VALUE_HASHTABLE: APIKEYS_BY_VALUE_HASHTABLE.with(|store| store.borrow().clone()),
        APIKEYS_BY_ID_HASHTABLE: APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        USERS_APIKEYS_HASHTABLE: USERS_APIKEYS_HASHTABLE.with(|store| store.borrow().clone()),
        // Add more state sections as needed
    }
}

pub fn snapshot_prestate() -> Option<EntireState> {
    let diff_webhooks = get_active_state_diff_webhooks();
    if diff_webhooks.is_empty() {
        return None
    }
    let before_state = snapshot_entire_state();
    Some(before_state)
}

pub fn snapshot_poststate(before_snapshot: Option<EntireState>, notes: Option<String>) {
    match before_snapshot {
        Some(before_snapshot) => {
            let after_snapshot = snapshot_entire_state();
            let diff = diff_entire_state(before_snapshot, after_snapshot);
            match diff {
                Some(diff) => {
                    fire_state_diff_webhooks(diff, notes);
                },
                None => ()
            }
        },
        None => ()
    }
}

pub fn diff_entire_state(before_snapshot: EntireState, after_snapshot: EntireState) -> Option<DriveStateDiffString> {
    // Create MessagePack diff (minimal size)
    let diff_data = match rmp_serde::to_vec_named(&Diff::serializable(&before_snapshot, &after_snapshot)) {
        Ok(data) => data,
        Err(e) => {
            ic_cdk::println!("Failed to serialize state diff: {}", e);
            Vec::new()
        }
    };

    if diff_data.len() <= 4 {  // Adjust this threshold based on testing
        return None;  // No meaningful difference, skip firing
    }
    
    // Convert diff to base64 for transmission if needed
    let diff_base64 = base64::encode(&diff_data);
    Some(DriveStateDiffString(diff_base64))
}