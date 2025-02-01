// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::core::state::{
    apikeys::state::state::{HASHTABLE_APIKEYS_BY_VALUE, HASHTABLE_APIKEYS_BY_ID},
    apikeys::types::{ApiKeyValue, ApiKey},
};
use crate::rest::apikeys::types::ErrorResponse;
use std::time::{SystemTime, UNIX_EPOCH};

use super::helpers::create_response;

// Add this helper function in your apikeys_handlers module
pub fn authenticate_request(req: &HttpRequest) -> Option<ApiKey> {
    // First extract the api key header
    let api_key_str = match req.headers().iter().find(|(k, _)| k == "api-key") {
        Some((_, value)) => value,
        None => return None,
    };

    // Convert to ApiKeyValue type
    let api_key_value = ApiKeyValue(api_key_str.to_string());

    // Look up the API key ID using the value
    let api_key_id = HASHTABLE_APIKEYS_BY_VALUE.with(|store| {
        store.borrow().get(&api_key_value).cloned()
    });

    let api_key_id = match api_key_id {
        Some(id) => id,
        None => return None,
    };

    // Look up the full API key using the ID
    let full_api_key = HASHTABLE_APIKEYS_BY_ID.with(|store| {
        store.borrow().get(&api_key_id).cloned()
    });

    // Check if key exists and validate expiration/revocation
    if let Some(key) = full_api_key {
        // Get current Unix timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
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
    let body = String::from_utf8(ErrorResponse::not_allowed().encode())
        .unwrap_or_else(|_| String::from("Unauthorized"));
    create_response(StatusCode::UNAUTHORIZED, body)
}