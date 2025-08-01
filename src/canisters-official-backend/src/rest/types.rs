// src/types.rs
use serde::{Deserialize, Serialize};
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;
use candid::Principal;
use serde_diff::SerdeDiff;
use std::{fmt, str::FromStr};

use crate::{core::{state::drives::state::state::UUID_CLAIMED, types::IDPrefix}, debug_log};

use super::auth::{seed_phrase_to_wallet_addresses, WalletAddresses};

pub type RouteHandler = for<'a, 'k, 'v> fn(&'a HttpRequest<'a>, &'a Params<'k, 'v>) 
    -> core::pin::Pin<Box<dyn core::future::Future<Output = HttpResponse<'static>> + 'a>>;



    
#[derive(Debug, Clone, Serialize)]
pub enum ApiResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> ApiResponse<'a, T> {
    pub fn ok(data: &'a T) -> Self {
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn unauthorized() -> Self {
        Self::err(401, "Unauthorized".to_string())
    }

    pub fn forbidden() -> Self {
        Self::err(403, "Forbidden".to_string())
    }

    pub fn bad_request(message: String) -> Self {
        Self::err(400, message)
    }

    pub fn server_error(message: String) -> Self {
        Self::err(500, message)
    }

    pub fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_else(|_| 
            // Explicitly specify the type parameter
            serde_json::to_vec(&ApiResponse::<()>::err(
                500, 
                "Failed to serialize response".to_string()
            ))
            .unwrap_or_default()
        )
    }
}

    // Add ValidationError struct
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub fn validate_unclaimed_uuid(id: &str) -> Result<(), ValidationError> {
    debug_log!("Validating unclaimed UUID: {}", id);

    // print out the UUID_CLAIMED value noting lifetimes
    debug_log!("UUID_CLAIMED: {:?}", UUID_CLAIMED);

    // check that this id isnt already claimed
    if UUID_CLAIMED.with(|claimed| claimed.borrow().contains_key(&id.to_string())) {
        return Err(ValidationError {
            field: "id is not unique".to_string(),
            message: format!("{} is already claimed", id),
        });
    }
    
    Ok(())
}

// Helper functions for ID validation
pub fn validate_id_string(id: &str, field_name: &str) -> Result<(), ValidationError> {
    // Check length
    if id.is_empty() {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} cannot be empty", field_name),
        });
    }
    
    if id.len() > 256 {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be 256 characters or less", field_name),
        });
    }

    Ok(())
}




pub fn validate_short_string(id: &str, field_name: &str) -> Result<(), ValidationError> {
    // Check max length only
    if id.len() > 256 {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be 256 characters or less", field_name),
        });
    }
    
    Ok(())
}


pub fn validate_user_id(user_id: &str) -> Result<(), ValidationError> {
    // Check basic string requirements first
    validate_id_string(user_id, "user_id")?;
    
    // Check if it starts with the correct prefix
    let user_prefix = IDPrefix::User.as_str();
    if !user_id.starts_with(user_prefix) {
        return Err(ValidationError {
            field: "user_id".to_string(),
            message: format!("User ID must start with '{}'", user_prefix),
        });
    }
    
    // Extract the ICP principal part
    let principal_str = &user_id[user_prefix.len()..];
    
    // Validate as ICP principal
    match Principal::from_text(principal_str) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError {
            field: "user_id".to_string(),
            message: "User ID contains an invalid ICP principal".to_string(),
        }),
    }
}

pub fn validate_drive_id(drive_id: &str) -> Result<(), ValidationError> {
    // Check basic string requirements first
    validate_id_string(drive_id, "drive_id")?;
    
    // Check if it starts with the correct prefix
    let drive_prefix = IDPrefix::Drive.as_str();
    if !drive_id.starts_with(drive_prefix) {
        return Err(ValidationError {
            field: "drive_id".to_string(),
            message: format!("Drive ID must start with '{}'", drive_prefix),
        });
    }
    
    // Extract the ICP principal part
    let principal_str = &drive_id[drive_prefix.len()..];
    
    // Validate as ICP principal
    match Principal::from_text(principal_str) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError {
            field: "drive_id".to_string(),
            message: "Drive ID contains an invalid ICP principal".to_string(),
        }),
    }
}

pub fn validate_evm_address(address: &str) -> Result<(), ValidationError> {
    // Check if empty
    if address.is_empty() {
        return Err(ValidationError {
            field: "evm_address".to_string(),
            message: "EVM address cannot be empty".to_string(),
        });
    }
    
    // Check prefix
    if !address.starts_with("0x") {
        return Err(ValidationError {
            field: "evm_address".to_string(),
            message: "EVM address must start with '0x'".to_string(),
        });
    }
    
    // Check length (0x + 40 hex chars)
    if address.len() != 42 {
        return Err(ValidationError {
            field: "evm_address".to_string(),
            message: "EVM address must be 42 characters long (including '0x')".to_string(),
        });
    }
    
    // Check that all characters after 0x are valid hex digits
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError {
            field: "evm_address".to_string(),
            message: "EVM address must contain only hexadecimal characters after '0x'".to_string(),
        });
    }
    
    Ok(())
}

pub fn validate_icp_principal(principal: &str) -> Result<(), ValidationError> {
    // Check if empty
    if principal.is_empty() {
        return Err(ValidationError {
            field: "icp_principal".to_string(),
            message: "ICP principal cannot be empty".to_string(),
        });
    }
    
    // Validate as ICP principal
    match Principal::from_text(principal) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError {
            field: "icp_principal".to_string(),
            message: "Invalid ICP principal format".to_string(),
        }),
    }
}

pub fn validate_seed_phrase(seed_phrase: &str) -> Result<WalletAddresses, ValidationError> {
    seed_phrase_to_wallet_addresses(seed_phrase).map_err(|err| {
        ValidationError {
            field: "seed_phrase".to_string(),
            message: err.message,
        }
    })
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    // Check if empty
    if email.is_empty() {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email cannot be empty".to_string(),
        });
    }
    
    // Check maximum length (RFC 5321 limits to 254 characters)
    if email.len() > 254 {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email must be 254 characters or less".to_string(),
        });
    }
    
    // Basic format validation: must contain exactly one @
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email must contain exactly one @ symbol".to_string(),
        });
    }
    
    let (local_part, domain) = (parts[0], parts[1]);
    
    // Local part cannot be empty
    if local_part.is_empty() {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email local part (before @) cannot be empty".to_string(),
        });
    }
    
    // Domain cannot be empty
    if domain.is_empty() {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email domain (after @) cannot be empty".to_string(),
        });
    }
    
    // Domain must contain at least one dot
    if !domain.contains('.') {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email domain must contain at least one dot".to_string(),
        });
    }
    
    // Domain cannot end with a dot
    if domain.ends_with('.') {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email domain cannot end with a dot".to_string(),
        });
    }
    
    // Check for illegal characters
    if local_part.chars().any(|c| !c.is_ascii_alphanumeric() && "!#$%&'*+-/=?^_`{|}~.".find(c).is_none()) {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email local part contains illegal characters".to_string(),
        });
    }
    
    if domain.chars().any(|c| !c.is_ascii_alphanumeric() && c != '.' && c != '-') {
        return Err(ValidationError {
            field: "email".to_string(),
            message: "Email domain contains illegal characters".to_string(),
        });
    }
    
    Ok(())
}

pub fn validate_url(url: &str, field_name: &str) -> Result<(), ValidationError> {
    // Check if empty
    if url.is_empty() {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} cannot be empty", field_name),
        });
    }
    
    // Check maximum length
    if url.len() > 2048 {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be 2048 characters or less", field_name),
        });
    }
    
    // URL must start with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must start with http:// or https://", field_name),
        });
    }
    
    // Check if URL has a domain part
    let url_without_scheme = if url.starts_with("https://") {
        &url[8..]
    } else {
        &url[7..]
    };
    
    // Domain part cannot be empty
    if url_without_scheme.is_empty() {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must contain a domain", field_name),
        });
    }
    
    // Domain part must contain at least one dot (except for localhost)
    if !url_without_scheme.starts_with("localhost") && !url_without_scheme.contains('.') {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} domain must contain at least one dot", field_name),
        });
    }
    
    Ok(())
}

pub fn validate_external_id(external_id: &str) -> Result<(), ValidationError> {
    // External IDs are simpler, just validate the string length
    if external_id.len() > 256 {
        return Err(ValidationError {
            field: "external_id".to_string(),
            message: "External ID must be 256 characters or less".to_string(),
        });
    }
    
    Ok(())
}

pub fn validate_external_payload(payload: &str) -> Result<(), ValidationError> {
    // External payloads have a max size of 8,192 characters
    if payload.len() > 8192 {
        return Err(ValidationError {
            field: "external_payload".to_string(),
            message: "External payload must be 8,192 characters or less".to_string(),
        });
    }
    
    Ok(())
}

// Validate URL length 
pub fn validate_url_endpoint(url: &str, field_name: &str) -> Result<(), ValidationError> {
    // Check length 
    if url.len() > 4096 {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be 4,096 characters or less", field_name),
        });
    }
    
    Ok(())
}

// Validate description
pub fn validate_description(description: &str, field_name: &str) -> Result<(), ValidationError> {
    // Check length
    if description.len() > 8192 {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be 8,192 characters or less", field_name),
        });
    }
    
    Ok(())
}