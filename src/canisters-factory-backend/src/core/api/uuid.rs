// src/core/api/uuid.rs

use crate::{core::{state::{api_keys::types::{ApiKeyProof, ApiKeyValue, AuthTypeEnum},  giftcards::{types::{DriveID}}}, types::{IDPrefix, UserID}}, debug_log};
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


pub fn generate_uuidv4(prefix: IDPrefix) -> String {
    let canister_id = ic_cdk::api::id().to_string();
    let current_time = ic_cdk::api::time();

    let entropy_input = format!("{}-{}", canister_id, current_time);
    let mut hasher = Sha256::new();
    hasher.update(entropy_input.as_bytes());
    let mut hash_bytes = hasher.finalize();

    // Take the first 16 bytes (128 bits) of the hash
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&hash_bytes[..16]);

    // Set UUID version to 4
    hash_bytes[6] = (hash_bytes[6] & 0x0f) | 0x40;
    // Set UUID variant bits to "10xx"
    hash_bytes[8] = (hash_bytes[8] & 0x3f) | 0x80;

    // Format bytes into UUID string format
    let pseudo_uuid = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3],
        hash_bytes[4], hash_bytes[5],
        hash_bytes[6], hash_bytes[7],
        hash_bytes[8], hash_bytes[9],
        hash_bytes[10], hash_bytes[11], hash_bytes[12], hash_bytes[13], hash_bytes[14], hash_bytes[15]
    );
    let pseudo_prefix_id = format!("{}{}", prefix.as_str(), pseudo_uuid);
    pseudo_prefix_id
}


pub fn generate_api_key() -> String {
    let input = generate_uuidv4(IDPrefix::ApiKey);
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

