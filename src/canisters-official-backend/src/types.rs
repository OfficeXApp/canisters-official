// src/types.rs
use serde::{Deserialize, Serialize};
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;
use candid::Principal;
use ethers::types::Address as EVMAddress;
use std::str::FromStr;

use crate::core::types::IDPrefix;

pub type RouteHandler = for<'a, 'k, 'v> fn(&'a HttpRequest<'a>, &'a Params<'k, 'v>) 
    -> core::pin::Pin<Box<dyn core::future::Future<Output = HttpResponse<'static>> + 'a>>;

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
    
    // Validate as EVM address
    match EVMAddress::from_str(address) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError {
            field: "evm_address".to_string(),
            message: "Invalid EVM address format".to_string(),
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