use candid::Principal;
// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::{api::types::AuthHeaderBySignatureSchema, state::{api_keys::{state::state::{debug_state,APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyValue}}, drives::state::state::CANISTER_ID}, types::PublicKeyICP}, debug_log};
use crate::rest::api_keys::types::ErrorResponse;
use ic_cdk::api::call::arg_data;

use super::helpers::create_response;

const SIGNATURE_EXPIRY_MS: u64 = 60_000; // 60 seconds in milliseconds



pub fn verify_signature(
    public_key: &PublicKeyICP,
    message: &[u8],
    signature: &[u8]
) -> Result<bool, String> {
    // // Convert the public key from hex string to bytes
    // let pk_bytes = hex::decode(&public_key.0)
    //     .map_err(|e| format!("Invalid public key hex: {}", e))?;

    // // Use ic_cdk::sign for verification
    // sign::verify_signature(&pk_bytes, message, signature)
    //     .map_err(|e| format!("Signature verification failed: {}", e))
}


pub fn validate_auth_signature_method(
    signature_auth: &AuthHeaderBySignatureSchema
) -> Result<bool, String> {
    // 1. Verify timestamp is within 60,000 ms (60 seconds)
    let current_time_ms = ic_cdk::api::time();
    let challenge_age_ms = current_time_ms - signature_auth.challenge.timestamp_ms;
    
    if challenge_age_ms > SIGNATURE_EXPIRY_MS {
        return Err("Signature challenge has expired".to_string());
    }

    // 2. Verify canister ID matches
    if signature_auth.challenge.drive_canister_id != CANISTER_ID {
        return Err("Invalid drive canister ID".to_string());
    }

    // 3. Verify signature using public key from challenge
    let challenge_bytes = serde_json::to_vec(&signature_auth.challenge)
        .map_err(|e| format!("Failed to serialize challenge: {}", e))?;

    match verify_signature(
        &signature_auth.challenge.user_icp_public_key,  // Use public key from challenge
        &challenge_bytes,
        &signature_auth.signature
    ) {
        Ok(is_valid) => {
            if !is_valid {
                return Err("Invalid signature".to_string());
            }
            Ok(true)
        },
        Err(e) => Err(format!("Signature verification failed: {}", e))
    }
}


// Add this helper function in your apikeys_handlers module
pub fn authenticate_request(req: &HttpRequest) -> Option<ApiKey> {
    // Extract the Authorization header
    let auth_header = match req.headers().iter().find(|(k, _)| k == "authorization") {
        Some((_, value)) => value,
        None => return None,
    };

    // Parse "Bearer <token>"
    let token = match auth_header.strip_prefix("Bearer ") {
        Some(token) => token.trim(),
        None => return None,
    };

    // Convert to ApiKeyValue type
    let api_key_value = ApiKeyValue(token.to_string());

    debug_log!("api_key_value: {}", api_key_value);
    debug_log!("Current state: {}", debug_state());

    // Look up the API key ID using the value
    let api_key_id = APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
        store.borrow().get(&api_key_value).cloned()
    });

    let api_key_id = match api_key_id {
        Some(id) => id,
        None => return None,
    };
    
    debug_log!("api_key_id: {}", api_key_id);

    // Look up the full API key using the ID
    let full_api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
        store.borrow().get(&api_key_id).cloned()
    });

    debug_log!("full_api_key: {}", full_api_key.clone().unwrap());

    // Check if key exists and validate expiration/revocation
    if let Some(key) = full_api_key {
        // Get current Unix timestamp
        let now = ic_cdk::api::time() as i64;
        
        debug_log!("key check - expires_at: {}, is_revoked: {}", key.expires_at, key.is_revoked);

        // Check if key is expired (expires_at > 0 and current time exceeds it)
        // or if key is revoked
        if (key.expires_at > 0 && now >= key.expires_at) || key.is_revoked {
            None
        } else {
            Some(key)
        }
    } else {
        None
    }
}

pub fn create_auth_error_response() -> HttpResponse<'static> {
    let body = String::from_utf8(ErrorResponse::unauthorized().encode())
        .unwrap_or_else(|_| String::from("Unauthorized"));
    create_response(StatusCode::UNAUTHORIZED, body)
}



pub fn create_raw_upload_error_response(error_msg: &str) -> HttpResponse<'static> {
    // Use the new `unauthorized_with_message` (or whichever method you create)
    let error_struct = ErrorResponse::err(400, error_msg.to_string());

    let body = String::from_utf8(error_struct.encode())
        .unwrap_or_else(|_| String::from("Bad Request"));

    create_response(StatusCode::BAD_REQUEST, body)
}