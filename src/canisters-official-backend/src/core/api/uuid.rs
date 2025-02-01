use crate::core::state::drive::state::state::GLOBAL_UUID_NONCE;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_unique_id() -> String {
    let canister_id = ic_cdk::api::id().to_string();          // Canister's unique ID
    let current_time = ic_cdk::api::time();                   // Nanoseconds timestamp
    let caller = ic_cdk::api::caller().to_string();           // Principal of the caller
    
    // Increment the counter for every call
    GLOBAL_UUID_NONCE.with(|counter| {
        let current_counter = counter.get();
        counter.set(current_counter + 1);

        // Create a unique string by combining deterministic inputs
        let input_string = format!("{}-{}-{}-{}", canister_id, current_time, caller, current_counter);

        // Use SHA256 to hash the input string and produce a compact, unique identifier
        let mut hasher = Sha256::new();
        hasher.update(input_string);
        format!("{:x}", hasher.finalize())
    })
}

pub fn generate_api_key() -> String {
    // Get input from generate_unique_id
    let input = generate_unique_id();
    
    // Get current timestamp in nanoseconds as salt
    let salt = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    
    // Combine input and salt
    let combined = format!("{}{}", input, salt);
    
    // Create hasher and feed in our combined string
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    
    // Get the hash result and encode it as base64
    let hash = hasher.finalize();
    BASE64.encode(hash)
}