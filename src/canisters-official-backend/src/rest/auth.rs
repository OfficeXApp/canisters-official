use candid::Principal;
// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::{state::api_keys::{state::state::{debug_state,APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue, AuthJsonDecoded, AuthTypeEnum}}, types::UserID}, debug_log};
use crate::rest::api_keys::types::ErrorResponse;
use ic_types::crypto::AlgorithmId;
use serde::{Deserialize, Serialize};
use ic_crypto_standalone_sig_verifier::{
    verify_basic_sig_by_public_key,
};
use ic_crypto_standalone_sig_verifier::user_public_key_from_bytes;
use super::helpers::create_response;


pub fn authenticate_request(req: &HttpRequest) -> Option<ApiKey> {
    // Extract the Authorization header
    let auth_header = match req.headers().iter().find(|(k, _)| k == "authorization") {
        Some((_, value)) => value,
        None => {
            debug_log!("No authorization header found");
            return None;
        },
    };

    // Parse "Bearer <token>"
    let btoa_token = match auth_header.strip_prefix("Bearer ") {
        Some(token) => token.trim(),
        None => {
            debug_log!("Authorization header not in Bearer format");
            return None;
        },
    };

    // Decode the base64 proof string
    let stringified_token = match base64::decode(btoa_token) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(json_str) => json_str,
            Err(e) => {
                debug_log!("Failed to decode btoa_token as UTF-8: {}", e);
                return None;
            },
        },
        Err(e) => {
            debug_log!("Failed to decode base64 btoa_token: {}", e);
            return None;
        },
    };

    // Parse the JSON proof
    let auth_json: AuthJsonDecoded = match serde_json::from_str(&stringified_token) {
        Ok(proof) => proof,
        Err(e) => {
            debug_log!("Failed to parse JSON proof: {}", e);
            return None;
        },
    };

    // Handle different authentication types
    match auth_json {
        AuthJsonDecoded::Signature(proof) => {
            // Check that it's actually the signature type
            if proof.auth_type != AuthTypeEnum::Signature {
                debug_log!("Auth type mismatch in signature proof");
                return None;
            }

            // Check timestamp is within 30 seconds
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            let challenge_time = proof.challenge.timestamp_ms;
            if now > challenge_time + 30_000 || now < challenge_time {
                debug_log!("Timestamp out of valid range");
                return None; // Timestamp too old or from the future
            }

            // Convert signature from array to bytes
            let signature_bytes = proof.signature.as_slice();
            
            // Convert the challenge to bytes (this is what was signed)
            let challenge_json = match serde_json::to_string(&proof.challenge) {
                Ok(json) => json,
                Err(e) => {
                    debug_log!("Failed to serialize challenge to JSON: {}", e);
                    return None;
                },
            };
            let challenge_bytes = challenge_json.as_bytes();

            // Get the public key from the challenge
            let public_key = &proof.challenge.user_icp_public_key;

            // Use the standalone-sig-verifier to verify the signature
            match verify_basic_sig_by_public_key(
                AlgorithmId::Ed25519,
                challenge_bytes,
                signature_bytes,
                public_key,
            ) {
                Ok(_) => {
                    debug_log!("Signature verification successful");

                    // Convert DER public key to principal string
                    let public_key_result = match user_public_key_from_bytes(public_key) {
                        Ok(result) => result,
                        Err(e) => {
                            debug_log!("Failed to parse public key: {:?}", e);
                            return None;
                        },
                    };

                    // Get the user public key
                    let user_public_key = public_key_result.0;
                    
                    // Calculate the principal
                    let principal = Principal::self_authenticating(&user_public_key.key);
                    let principal_text = principal.to_text();

                    debug_log!("Successfully authenticated principal: {}", principal_text.clone());

                    // Authentication successful, return an API key
                    Some(ApiKey {
                        id: ApiKeyID(format!("sig_auth_{}", now)),
                        value: ApiKeyValue(format!("signature_auth_{}", principal_text.clone())),
                        user_id: UserID(principal_text.clone()),
                        name: format!("Signature Authenticated User {}", principal_text.clone()),
                        created_at: now, 
                        expires_at: -1, 
                        is_revoked: false,
                    })
                },
                Err(e) => {
                    debug_log!("Signature verification failed: {:?}", e);
                    None
                },
            }
        },
        AuthJsonDecoded::ApiKey(proof) => {
            // Verify it's the API key type
            if proof.auth_type != AuthTypeEnum::ApiKey {
                debug_log!("Auth type mismatch in API key proof");
                return None;
            }
            
            // Look up the API key value from the proof
            let api_key_value = proof.value;
            debug_log!("Looking up API key from JSON payload: {}", api_key_value.0);
            
            // Look up the API key ID using the value
            let api_key_id = APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
                store.borrow().get(&api_key_value).cloned()
            });
            
            if let Some(api_key_id) = api_key_id {
                debug_log!("Found api_key_id: {}", api_key_id);
                
                // Look up the full API key using the ID
                let full_api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
                    store.borrow().get(&api_key_id).cloned()
                });
                
                // Check if key exists and validate expiration/revocation
                if let Some(key) = full_api_key {
                    // Get current Unix timestamp
                    let now = ic_cdk::api::time() as i64;
                    
                    debug_log!("key check - expires_at: {}, is_revoked: {}", key.expires_at, key.is_revoked);
                    
                    // Return the key if it's valid (not expired and not revoked)
                    if (key.expires_at <= 0 || now < key.expires_at) && !key.is_revoked {
                        return Some(key);
                    }
                }
            }
            
            debug_log!("API key authentication failed");
            None
        }
    }
}

// pub fn authenticate_request(req: &HttpRequest) -> Option<ApiKey> {
//     // Extract the Authorization header
//     let auth_header = match req.headers().iter().find(|(k, _)| k == "authorization") {
//         Some((_, value)) => value,
//         None => return None,
//     };

//     // Parse "Bearer <token>"
//     let token = match auth_header.strip_prefix("Bearer ") {
//         Some(token) => token.trim(),
//         None => return None,
//     };

//     // Convert to ApiKeyValue type
//     let api_key_value = ApiKeyValue(token.to_string());

//     debug_log!("api_key_value: {}", api_key_value);
//     debug_log!("Current state: {}", debug_state());

//     // Look up the API key ID using the value
//     let api_key_id = APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
//         store.borrow().get(&api_key_value).cloned()
//     });

//     let api_key_id = match api_key_id {
//         Some(id) => id,
//         None => return None,
//     };
    
//     debug_log!("api_key_id: {}", api_key_id);

//     // Look up the full API key using the ID
//     let full_api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
//         store.borrow().get(&api_key_id).cloned()
//     });

//     debug_log!("full_api_key: {}", full_api_key.clone().unwrap());

//     // Check if key exists and validate expiration/revocation
//     if let Some(key) = full_api_key {
//         // Get current Unix timestamp
//         let now = ic_cdk::api::time() as i64;
        
//         debug_log!("key check - expires_at: {}, is_revoked: {}", key.expires_at, key.is_revoked);

//         // Check if key is expired (expires_at > 0 and current time exceeds it)
//         // or if key is revoked
//         if (key.expires_at > 0 && now >= key.expires_at) || key.is_revoked {
//             None
//         } else {
//             Some(key)
//         }
//     } else {
//         None
//     }
// }


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