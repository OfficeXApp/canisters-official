// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::state::api_keys::{state::state::{debug_state,APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyValue}}, debug_log};
use crate::rest::api_keys::types::ErrorResponse;
use ic_types::Principal;
use ic_types::crypto::AlgorithmId;
use ic_crypto_standalone_sig_verifier::{
    verify_basic_sig_by_public_key, 
    sign_utils::user_public_key_from_bytes
};

use super::helpers::create_response;

#[derive(Deserialize)]
struct SignatureProof {
    auth_type: String,
    challenge: Challenge,
    signature: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
struct Challenge {
    timestamp_ms: u64,
    drive_canister_id: String,
    user_icp_public_key: Vec<u8>,
}

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
    let token = match auth_header.strip_prefix("Bearer ") {
        Some(token) => token.trim(),
        None => {
            debug_log!("Authorization header not in Bearer format");
            return None;
        },
    };

    // Decode the base64 proof string
    let proof_json = match base64::decode(token) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(json_str) => json_str,
            Err(e) => {
                debug_log!("Failed to decode token as UTF-8: {}", e);
                return None;
            },
        },
        Err(e) => {
            debug_log!("Failed to decode base64 token: {}", e);
            return None;
        },
    };

    // Parse the JSON proof
    let proof: SignatureProof = match serde_json::from_str(&proof_json) {
        Ok(proof) => proof,
        Err(e) => {
            debug_log!("Failed to parse JSON proof: {}", e);
            return None;
        },
    };

    // Check auth_type is "Signature"
    if proof.auth_type != "Signature" {
        debug_log!("Auth type is not 'Signature'");
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

            debug_log!("Successfully authenticated principal: {}", principal_text);

            // Authentication successful, return an API key
            Some(ApiKey {
                id: format!("sig_auth_{}", now),
                value: ApiKeyValue(format!("signature_auth_{}", principal_text)),
                user_id: principal_text,
                name: format!("Signature Authenticated User {}", principal_text),
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