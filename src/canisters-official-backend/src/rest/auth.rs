use candid::Principal;
use ed25519_dalek::SigningKey;
// src/rest/auth.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use crate::{core::{api::uuid::format_user_id, state::api_keys::{state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue, AuthJsonDecoded, AuthTypeEnum}}, types::UserID}, debug_log, rest::helpers::update_last_online_at};
use crate::rest::api_keys::types::ErrorResponse;
use ic_types::crypto::AlgorithmId;
use bip39::{Mnemonic, Language};
use tiny_keccak::{Keccak, Hasher};
use serde::{Deserialize, Serialize};
use ic_crypto_standalone_sig_verifier::{
    verify_basic_sig_by_public_key,
};
use super::helpers::create_response;


pub fn authenticate_request(req: &HttpRequest) -> Option<ApiKey> {
    // // Extract the Authorization header
    // let auth_header = match req.headers().iter().find(|(k, _)| k == "authorization") {
    //     Some((_, value)) => value,
    //     None => {
    //         debug_log!("No authorization header found");
    //         return None;
    //     },
    // };

    // // debug_log!("auth_header: {}", auth_header);

    // // Parse "Bearer <token>"
    // let btoa_token = match auth_header.strip_prefix("Bearer ") {
    //     Some(token) => token.trim(),
    //     None => {
    //         debug_log!("Authorization header not in Bearer format");
    //         return None;
    //     },
    // };

    // Try to get the token from the Authorization header first
    let mut btoa_token: Option<String> = None;
    
    // Check Authorization header
    if let Some((_, auth_value)) = req.headers().iter().find(|(k, _)| k == "authorization") {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            btoa_token = Some(token.trim().to_string());
            debug_log!("Found token in Authorization header");
        } else {
            debug_log!("Authorization header not in Bearer format");
        }
    }
    
    // If no token from header, try query parameter
    if btoa_token.is_none() {
        if let Some(query_string) = req.url().split('?').nth(1) {
            // Parse the query string
            for param in query_string.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    if key == "auth" {
                        debug_log!("Found auth query parameter: {}", value);
                        btoa_token = Some(value.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    // If no token found in either place, return None
    let btoa_token = match btoa_token {
        Some(token) => token,
        None => {
            debug_log!("No authentication token found in header or query parameter");
            return None;
        }
    };
    

    // debug_log!("btoa_token: {}", btoa_token);

    let padded_token = match btoa_token.len() % 4 {
        0 => btoa_token.to_string(),
        n => format!("{}{}", btoa_token, "=".repeat(4 - n))
    };


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


    // Parse the JSON proof
    let auth_json: AuthJsonDecoded = match serde_json::from_str(&stringified_token) {
        Ok(proof) => proof,
        Err(e) => {
            debug_log!("Failed to parse JSON proof: {}", e);
            return None;
        },
    };

    // debug_log!("auth_json: {:?}", auth_json);

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

                    update_last_online_at(&format_user_id(&computed_principal.clone()));

                    // Create and return an API key based on the computed principal.
                    Some(ApiKey {
                        id: ApiKeyID(format!("sig_auth_{}", now)),
                        value: ApiKeyValue(format!("signature_auth_{}", computed_principal)),
                        user_id: format_user_id(&computed_principal.clone()),
                        name: format!("Signature Authenticated User {}", computed_principal),
                        private_note: None,
                        created_at: now,
                        begins_at: 0,
                        expires_at: -1,
                        is_revoked: false,
                        labels: vec![],
                        external_id: None,
                        external_payload: None,
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
            // debug_log!("Looking up API key from JSON payload: {}", api_key_value);
            
            // Look up the API key ID using the value
            let api_key_id = APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
                store.borrow().get(&api_key_value).map(|data| data.clone())
            });
            
            if let Some(api_key_id) = api_key_id {
                // debug_log!("Found api_key_id: {}", api_key_id);
                
                // Look up the full API key using the ID
                let full_api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
                    store.borrow().get(&api_key_id).map(|key| key.clone())
                });
                
                // Check if key exists and validate expiration/revocation
                if let Some(key) = full_api_key {
                    // Get current Unix timestamp
                    let now = (ic_cdk::api::time() / 1_000_000) as i64;
                    
                    debug_log!("Successfully authenticated user: {}", key.user_id.clone());
                    
                    // Return the key if it's valid (not expired and not revoked), and begins time is past
                    if (key.expires_at <= 0 || now < key.expires_at) && !key.is_revoked && key.begins_at <= (ic_cdk::api::time() / 1_000_000) {

                        update_last_online_at(&key.user_id);
                        return Some(key);
                    }
                }
            }
            
            debug_log!("API key authentication failed");
            None
        }
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddresses {
    pub icp_principal: String,
    pub evm_public_address: String,
}

#[derive(Debug, Clone)]
pub struct SeedPhraseError {
    pub message: String,
}

/// Converts a BIP39 seed phrase to ICP principal and EVM address
/// 
/// This function handles the cryptographic derivation of blockchain addresses
/// from a standard mnemonic seed phrase.
pub fn seed_phrase_to_wallet_addresses(seed_phrase: &str) -> Result<WalletAddresses, SeedPhraseError> {
    // Validate the mnemonic phrase
    let mnemonic = match Mnemonic::parse_in(Language::English, seed_phrase) {
        Ok(m) => m,
        Err(_) => {
            return Err(SeedPhraseError {
                message: "Invalid mnemonic seed phrase".to_string(),
            });
        }
    };

    // Generate the seed from the mnemonic
    let seed_bytes = mnemonic.to_seed("");
    
    // ---- ICP Principal Generation ----
    
    // Use the first 32 bytes of the seed for the Ed25519 key
    // We need to copy into a fixed-size array for the SigningKey::from_bytes method
    let mut identity_seed = [0u8; 32];
    identity_seed.copy_from_slice(&seed_bytes[0..32]);
    
    // Create Ed25519 keypair from the seed
    let signing_key = SigningKey::from_bytes(&identity_seed);
    
    // Get the verifying key (public key)
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_bytes();
    
    // To compute the canonical principal,
    // convert the raw public key into DER format by prepending the header
    let der_header: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];
    let mut der_key = Vec::with_capacity(44);
    der_key.extend_from_slice(&der_header);
    der_key.extend_from_slice(&public_key_bytes);
    
    // Compute the self-authenticating principal
    let principal = Principal::self_authenticating(&der_key);
    let icp_principal = principal.to_text();
    
    // ---- EVM Address Generation ----
    
    // For EVM, we need to derive a secp256k1 key
    // For simplicity, we'll use the same seed but with different derivation path logic
    // For a proper implementation, you should use a full BIP32/BIP44 derivation
    
    // Generate a private key for Ethereum (using a different part of the seed)
    // In a real implementation, you should use BIP32/44 derivation paths
    let eth_private_key = &seed_bytes[32..64];
    
    // Derive the Ethereum public key (this is simplified)
    // In a real implementation, you would derive the secp256k1 public key
    // For now, we'll create a mock public key derivation
    let mut eth_public_key = [0u8; 65];
    // This would be a real secp256k1 derivation in a full implementation
    eth_public_key[0] = 4; // Uncompressed key prefix
    eth_public_key[1..33].copy_from_slice(&eth_private_key);
    // Normally the Y coordinate would go here, but we're simplifying
    
    // Create Ethereum address: keccak256(public_key)[12:32]
    // We only use the X coordinate part in this simplified version
    let mut keccak = Keccak::v256();
    let mut eth_hash = [0u8; 32];
    keccak.update(&eth_public_key[1..33]);
    keccak.finalize(&mut eth_hash);
    
    // Take the last 20 bytes of the hash to form the address
    let evm_address = format!("0x{}", hex::encode(&eth_hash[12..32]));
    
    Ok(WalletAddresses {
        icp_principal,
        evm_public_address: evm_address,
    })
}