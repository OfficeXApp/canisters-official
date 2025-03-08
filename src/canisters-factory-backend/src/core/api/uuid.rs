// src/core/api/uuid.rs

use crate::{core::{state::{api_keys::types::{ApiKeyProof, ApiKeyValue, AuthTypeEnum},  giftcards::{state::state::{GLOBAL_UUID_NONCE}, types::{DriveID}}}, types::{IDPrefix, UserID}}, debug_log};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::{fmt, time::UNIX_EPOCH};
use serde::{Deserialize, Serialize};



pub fn format_user_id(principal_string: &str) -> UserID {
    UserID(format!("{}{}", IDPrefix::User.as_str(), principal_string))
}
pub fn format_drive_id(principal_string: &str) -> DriveID {
    DriveID(format!("{}{}", IDPrefix::Drive.as_str(), principal_string))
}


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
    let hash = hasher.finalize();
    
    let api_key_inner_value = hex::encode(hash);

    let api_key_proof = ApiKeyProof {
        auth_type: AuthTypeEnum::Api_Key,
        value: ApiKeyValue(api_key_inner_value.to_string()),
    };

    debug_log!("API Key Proof: {:?}", api_key_proof);
    
    // Serialize to JSON
    let json_payload = serde_json::to_string(&api_key_proof)
        .unwrap_or_else(|_| String::from("{}"));

    debug_log!("json_payload: {}", json_payload);
    
    // Base64 encode the JSON
    base64::encode(json_payload)
}

