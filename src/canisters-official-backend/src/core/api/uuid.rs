// src/core/api/uuid.rs

use crate::core::{state::{api_keys::types::{ApiKeyProof, ApiKeyValue, AuthTypeEnum}, directory::types::ShareTrackID, drives::{state::state::{DRIVE_STATE_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, GLOBAL_UUID_NONCE}, types::{DriveID, DriveStateDiffString, StateChecksum}}}, types::{IDPrefix, UserID}};
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
    
    // Serialize to JSON
    let json_payload = serde_json::to_string(&api_key_proof)
        .unwrap_or_else(|_| String::from("{}"));
    
    // Base64 encode the JSON
    base64::encode(json_payload)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShareTrackHash(pub String);

impl fmt::Display for ShareTrackHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ShareTrackHashData {
    id: String,
    from_user: String,
}

/// Generates a unique ShareTrackID and a ShareTrackHash for a user
/// 
/// The hash contains the ID and user information encoded in base64
/// which can be safely appended to URL parameters
pub fn generate_share_track_hash(user_id: &UserID) -> (ShareTrackID, ShareTrackHash) {
    // Generate a unique ID for this share track
    let share_track_id = generate_unique_id(IDPrefix::ShareTrackID, "");
    
    // Create the hash data object
    let hash_data = ShareTrackHashData {
        id: share_track_id.clone(),
        from_user: user_id.0.clone(),
    };
    
    // Serialize to JSON and encode with base64
    let json_data = serde_json::to_string(&hash_data)
        .expect("Failed to serialize share track data");
    let hash = BASE64.encode(json_data.as_bytes());
    
    (ShareTrackID(share_track_id), ShareTrackHash(hash))
}

/// Function to decode a share track hash back to its components
pub fn decode_share_track_hash(hash: &ShareTrackHash) -> (ShareTrackID, UserID) {
    // Attempt to decode the base64 string
    let decoded_bytes = match BASE64.decode(hash.0.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return (ShareTrackID(String::new()), UserID(String::new())),
    };
    
    // Convert bytes to string
    let json_str = match String::from_utf8(decoded_bytes) {
        Ok(str) => str,
        Err(_) => return (ShareTrackID(String::new()), UserID(String::new())),
    };
    
    // Parse the JSON into our struct
    match serde_json::from_str::<ShareTrackHashData>(&json_str) {
        Ok(hash_data) => (ShareTrackID(hash_data.id), UserID(hash_data.from_user)),
        Err(_) => (ShareTrackID(String::new()), UserID(String::new())),
    }
}
