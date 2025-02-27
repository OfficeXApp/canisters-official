use candid::Principal;
// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::{api::uuid::compile_user_id, state::api_keys::{state::state::{debug_state,APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue, AuthJsonDecoded, AuthTypeEnum}}, types::UserID}, debug_log};
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

    debug_log!("btoa_token: {}", btoa_token);

    let padded_token = match btoa_token.len() % 4 {
        0 => btoa_token.to_string(),
        n => format!("{}{}", btoa_token, "=".repeat(4 - n))
    };

    debug_log!("padded_token: {}", padded_token);

    // Decode the base64 proof string
    let stringified_token = match base64::decode(padded_token) {
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

    debug_log!("stringified_token: {}", stringified_token);

    // Parse the JSON proof
    let auth_json: AuthJsonDecoded = match serde_json::from_str(&stringified_token) {
        Ok(proof) => proof,
        Err(e) => {
            debug_log!("Failed to parse JSON proof: {}", e);
            return None;
        },
    };

    debug_log!("auth_json: {:?}", auth_json);

    // Handle different authentication types
    match auth_json {
        AuthJsonDecoded::Signature(proof) => {
            if proof.auth_type != AuthTypeEnum::Signature {
                debug_log!("Auth type mismatch in signature proof");
                return None;
            }

            // Check challenge timestamp (must be within 30 seconds), convert ns to ms
            let now = ic_cdk::api::time() as u64 / 1_000_000;
            if now > proof.challenge.timestamp_ms + 30_000 {
                debug_log!("Signature challenge expired");
                return None;
            }

            // Serialize the challenge as was signed.
            let challenge_json = serde_json::to_string(&proof.challenge).ok()?;
            let challenge_bytes = challenge_json.as_bytes();

            // The raw public key (32 bytes) as provided in the challenge.
            let public_key_bytes = &proof.challenge.self_auth_principal;
            if public_key_bytes.len() != 32 {
                debug_log!("Expected 32-byte raw public key, got {} bytes", public_key_bytes.len());
                return None;
            }

            // Verify the signature using the provided raw key.
            match verify_basic_sig_by_public_key(
                AlgorithmId::Ed25519,
                challenge_bytes,
                proof.signature.as_slice(),
                public_key_bytes,
            ) {
                Ok(_) => {
                    debug_log!("Signature verification successful");

                    // To compute the canonical principal that matches getPrincipal(),
                    // first convert the raw public key into DER format by prepending the header.
                    let der_header: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];
                    let mut der_key = Vec::with_capacity(44);
                    der_key.extend_from_slice(&der_header);
                    der_key.extend_from_slice(public_key_bytes);

                    // Compute the canonical principal using the DER-encoded key.
                    let computed_principal = Principal::self_authenticating(&der_key).to_text();

                    // Compare with the canonical_principal included in the challenge.
                    if computed_principal != proof.challenge.canonical_principal {
                        debug_log!(
                            "Mismatch between computed and provided canonical principal: {} vs {}",
                            computed_principal,
                            proof.challenge.canonical_principal
                        );
                        return None;
                    }
                    debug_log!("Successfully authenticated user: {}", computed_principal);

                    // Create and return an API key based on the computed principal.
                    Some(ApiKey {
                        id: ApiKeyID(format!("sig_auth_{}", now)),
                        value: ApiKeyValue(format!("signature_auth_{}", computed_principal)),
                        user_id: compile_user_id(&computed_principal.clone()),
                        name: format!("Signature Authenticated User {}", computed_principal),
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
            let api_key_value = ApiKeyValue(btoa_token.to_string());
            debug_log!("Looking up API key from JSON payload: {}", api_key_value);
            
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

