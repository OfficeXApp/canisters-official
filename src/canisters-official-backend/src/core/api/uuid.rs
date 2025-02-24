// src/core/api/uuid.rs

use crate::core::{state::{directory::types::ShareTrackID, drives::{state::state::{DRIVE_STATE_DIFF_CHECKSUM, GLOBAL_UUID_NONCE}, types::{DriveStateDiffChecksum, DriveStateDiffString}}}, types::{IDPrefix, UserID}};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::{fmt, time::UNIX_EPOCH};
use serde::{Deserialize, Serialize};

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
    
    hex::encode(hash)
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

#[derive(Debug, Serialize, Deserialize)]
struct StateDiffChecksumShape {
    timestamp_ns: u64,
    diff_string: DriveStateDiffString,
}

/// Generates a pseudo-checksum for state diff data
///
/// This is not a cryptographic checksum but rather an encoded representation
/// of the diff data along with a timestamp for uniqueness
pub fn update_checksum_for_state_diff(diff_string: DriveStateDiffString) {
    // Get current timestamp in nanoseconds
    let timestamp_ns = ic_cdk::api::time();
    
    // Create the checksum data object
    let checksum_data = StateDiffChecksumShape {
        timestamp_ns,
        diff_string,
    };
    
    // Serialize to JSON and encode with base64
    let json_data = serde_json::to_string(&checksum_data)
        .expect("Failed to serialize state diff checksum data");
    
    // Use the same base64 encoding from your UUID module
    let new_checksum = DriveStateDiffChecksum(BASE64.encode(json_data.as_bytes()));

    // Update checksum
    DRIVE_STATE_DIFF_CHECKSUM.with(|checksum| {
        *checksum.borrow_mut() = new_checksum;
    });
}