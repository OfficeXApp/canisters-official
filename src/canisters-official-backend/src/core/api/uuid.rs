// src/core/api/uuid.rs
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use crate::core::{state::drives::state::state::GLOBAL_UUID_NONCE, types::IDPrefix};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::time::{UNIX_EPOCH};

use super::types::{AuthHeaderByApiKeySchema, AuthType};


pub fn generate_unique_id(prefix: IDPrefix, suffix: &str) -> String {
    let drive_id = ic_cdk::api::id().to_string();          // Canister's unique ID
    let current_time = ic_cdk::api::time();                   // Nanoseconds timestamp
    let caller = ic_cdk::api::caller().to_string();           // Principal of the caller
    
    // Increment the counter for every call
    GLOBAL_UUID_NONCE.with(|counter| {
        let current_counter = counter.get();
        counter.set(current_counter + 1);

        // Create a unique string by combining deterministic inputs
        let input_string = format!("{}-{}-{}-{}", drive_id, current_time, caller, current_counter);

        // Use SHA256 to hash the input string and produce a compact, unique identifier
        let mut hasher = Sha256::new();
        hasher.update(input_string);
        format!("{}{:x}{}", prefix.as_str(), hasher.finalize(), suffix)
    })
}

pub fn generate_api_key() -> String {
    let input = generate_unique_id(IDPrefix::ApiKey, "");
    let salt = ic_cdk::api::time();
    let combined = format!("{}{}", input, salt);
    
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    // Create the payload object
    let payload = AuthHeaderByApiKeySchema {
        auth_type: AuthType::ApiKey,
        hash: hash,
    };

    // Convert to JSON string and then to base64
    let json_string = serde_json::to_string(&payload)
        .expect("Failed to serialize payload");
    
    base64.encode(json_string)
}