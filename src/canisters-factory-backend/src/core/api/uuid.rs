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
    use tiny_keccak::{Hasher, Keccak};
    
    let drive_id = ic_cdk::api::id().to_string();
    let current_time = ic_cdk::api::time();
    
    GLOBAL_UUID_NONCE.with(|counter| {
        let current_counter = counter.get();
        counter.set(current_counter + 1);
        
        // Create input string with our entropy sources (removed caller)
        let input_string = format!("{}-{}-{}", drive_id, current_time, current_counter);
        
        // Use Keccak-256 (from tiny-keccak crate you already have)
        let mut keccak = Keccak::v256();
        let mut hash = [0u8; 32];
        keccak.update(input_string.as_bytes());
        keccak.finalize(&mut hash);
        
        // Take only first 10 bytes (20 hex chars) for an even shorter ID
        let shortened = &hash[0..10];
        
        // Convert to hex
        let hex_id = hex::encode(shortened);
        
        // Format the ID with prefix and suffix
        format!("{}{}{}", prefix.as_str(), hex_id, suffix)
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

