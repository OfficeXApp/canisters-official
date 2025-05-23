// src/core/api/uuid.rs

use crate::{core::{state::{api_keys::types::{ApiKeyProof, ApiKeyValue, AuthTypeEnum}, directory::types::ShareTrackID, drives::{state::state::{DRIVE_STATE_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, NONCE_UUID_GENERATED, UUID_CLAIMED}, types::{DriveID, DriveStateDiffString, StateChecksum}}}, types::{IDPrefix, UserID}}, debug_log};
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
    
    // Get and increment the nonce
    let nonce = NONCE_UUID_GENERATED.with(|counter| {
        let current = *counter.borrow().get(); // Get the current value
        counter.borrow_mut().set(current + 1); // Set the incremented value
        current
    });

    // Include nonce in entropy input
    let entropy_input = format!("{}-{}-{}", canister_id, current_time, nonce);
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
    
    // Debug log with nonce for troubleshooting
    debug_log!("Generated UUID with nonce {}: {}", nonce, pseudo_prefix_id);
    
    pseudo_prefix_id
}

pub fn mark_claimed_uuid(uuid: &str) {
    UUID_CLAIMED.with(|claimed| {
        claimed.borrow_mut().insert(uuid.clone().to_string(), true);
    });
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
        auth_type: AuthTypeEnum::ApiKey,
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
    let share_track_id = generate_uuidv4(IDPrefix::ShareTrackID);
    // we wont mark_claimed_uuid cuz we expect multiple uses of the same id since its tracking aggregates. also can be responsibility of webhook / 3rd party analytics
    
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
