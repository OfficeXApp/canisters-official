use candid::Principal;
// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::{state::api_keys::{state::state::{debug_state,APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue}}, types::{ UserID}}, debug_log};
use crate::rest::api_keys::types::ErrorResponse;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64Standard };
use serde::{Deserialize, Serialize};
use super::helpers::create_response;

#[derive(Deserialize)]
struct SignatureRequest {
    content: SignatureContent,
    sender_pubkey: Vec<u8>,
    sender_sig: Vec<u8>
}

#[derive(Deserialize, Serialize)]
struct SignatureContent {
    ingress_expiry: u64,
    sender: String
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthPrefixEnum {
    ApiKey,
    Signature,
}
impl AuthPrefixEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthPrefixEnum::ApiKey => "ApiKey_",
            AuthPrefixEnum::Signature => "Signature_",
        }
    }
}

#[derive(Debug)]
pub struct ParsedAuth {
    pub auth_type: AuthPrefixEnum,
    pub value: String,
}


// Add this helper function in your apikeys_handlers module
pub fn authenticate_request(req: &HttpRequest) -> Option<ApiKey> {
    // Extract the Authorization header
    let auth_header = match req.headers().iter().find(|(k, _)| k == "authorization") {
        Some((_, value)) => value,
        None => return None,
    };

    let parsed_auth = parse_auth_header_value(auth_header).unwrap();

    match parsed_auth.auth_type {
        AuthPrefixEnum::ApiKey => {
            // Look up the API key ID using the value
            let api_key_id = APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
                store.borrow().get(&ApiKeyValue(parsed_auth.value)).cloned()
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
        AuthPrefixEnum::Signature => {
            // Verify signature and get principal
            if let Some(principal) = verify_signature(&parsed_auth) {
                let principal_str = principal.to_string();
                
                // Create mock API key using the principal
                let api_key = ApiKey {
                    id: ApiKeyID(principal_str.clone()),
                    value: ApiKeyValue(parsed_auth.value.clone()), // Use raw signature as value
                    user_id: UserID(principal_str.clone()),
                    name: principal_str,
                    created_at: (ic_cdk::api::time() - 60_000_000_000) as u64, // 60 seconds ago
                    expires_at: -1,
                    is_revoked: false,
                };

                Some(api_key)
            } else {
                None
            }
        }
    }

}


pub fn parse_auth_header_value(auth_header_value: &str) -> Result<ParsedAuth, &'static str> {
    // Strip "Bearer " prefix
    let raw_bearer = auth_header_value.strip_prefix("Bearer ")
        .ok_or("Authentication header must start with 'Bearer '")?;

    // Decode base64
    let decoded_bearer = Base64Standard.decode(raw_bearer)
        .map_err(|_| "Failed to decode base64 bearer token")?;
    
    // Rest of your code remains the same
    let decoded_str = String::from_utf8(decoded_bearer)
        .map_err(|_| "Invalid UTF-8 in decoded token")?;

    debug_log!("Decoded auth header: {}", decoded_str);

    if decoded_str.starts_with(AuthPrefixEnum::ApiKey.as_str()) {
        Ok(ParsedAuth {
            auth_type: AuthPrefixEnum::ApiKey,
            value: auth_header_value.to_string()
        })
    } else if decoded_str.starts_with(AuthPrefixEnum::Signature.as_str()) {
        Ok(ParsedAuth {
            auth_type: AuthPrefixEnum::Signature,
            value: decoded_str[AuthPrefixEnum::Signature.as_str().len()..].to_string(),
        })
    } else {
        Err("Invalid authentication type prefix")
    }
}

pub fn verify_signature(parsed_auth: &ParsedAuth) -> Option<Principal> {
    // Decode the base64 signature data
    let signature_data = match serde_json::from_str::<SignatureRequest>(&parsed_auth.value) {
        Ok(data) => data,
        Err(_) => return None,
    };

    // 1. Verify timestamp is within acceptable range
    let current_time = ic_cdk::api::time();
    if signature_data.content.ingress_expiry < current_time {
        return None;
    }

    // 2. Parse the public key
    let (public_key, _) = match user_public_key_from_bytes(&signature_data.sender_pubkey) {
        Ok(key) => key,
        Err(_) => return None,
    };

    // 3. Verify the sender principal matches the public key
    let derived_principal = Principal::self_authenticating(&public_key.key);
    let sender_principal = match Principal::from_text(&signature_data.content.sender) {
        Ok(principal) => principal,
        Err(_) => return None,
    };

    if derived_principal != sender_principal {
        return None;
    }

    // 4. Reconstruct the message with domain separator
    let domain_sep = b"\x0Aic-request";
    let content_bytes = match serde_json::to_vec(&signature_data.content) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };
    let full_message = [domain_sep, &content_bytes].concat();

    // 5. Verify the signature based on algorithm
    let is_valid = match public_key.algorithm_id {
        AlgorithmId::EcdsaSecp256k1 => {
            match secp256k1::api::verify(
                &signature_data.sender_sig,
                &full_message,
                &public_key.key
            ) {
                Ok(valid) => valid,
                Err(_) => false,
            }
        },
        _ => false, // Unsupported algorithm
    };

    if is_valid {
        Some(sender_principal)
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


