// src/types.rs
use serde::{Deserialize, Serialize};
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;
use candid::Principal;
use serde_diff::SerdeDiff;
use std::{fmt, str::FromStr};

use crate::core::types::IDPrefix;

pub type RouteHandler = for<'a, 'k, 'v> fn(&'a HttpRequest<'a>, &'a Params<'k, 'v>) 
    -> core::pin::Pin<Box<dyn core::future::Future<Output = HttpResponse<'static>> + 'a>>;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]

pub enum UpsertActionTypeEnum {
    Create,
    Update,
}
impl fmt::Display for UpsertActionTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpsertActionTypeEnum::Create => write!(f, "CREATE"),
            UpsertActionTypeEnum::Update => write!(f, "UPDATE"),
        }
    }
}


    
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