// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::state::api_keys::{state::state::{debug_state,APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyValue}}, debug_log};
use crate::rest::api_keys::types::ErrorResponse;

use super::helpers::create_response;

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