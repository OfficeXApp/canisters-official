// src/core/api/factory.rs

use crate::{core::{state::drives::state::state::{DRIVE_ID, RECENT_DEPLOYMENTS, URL_ENDPOINT}, types::UserID}, debug_log, rest::organization::types::{ RedeemOrgRequestBody, RedeemOrgResponseData}, LOCAL_DEV_MODE};
use candid::{CandidType, Encode};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformArgs, TransformContext, TransformFunc
};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument, CanisterSettings
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use crate::core::state::drives::types::{DriveID, FactorySpawnHistoryRecord};


// Define an error type for factory spawn operations
#[derive(Debug)]
pub enum FactorySpawnError {
    CanisterCreationFailed(String),
    InstallCodeFailed(String),
    HttpCallFailed(String),
    DeserializationFailed(String),
}

impl fmt::Display for FactorySpawnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FactorySpawnError::CanisterCreationFailed(msg) => write!(f, "Canister creation failed: {}", msg),
            FactorySpawnError::InstallCodeFailed(msg) => write!(f, "Install code failed: {}", msg),
            FactorySpawnError::HttpCallFailed(msg) => write!(f, "HTTP call failed: {}", msg),
            FactorySpawnError::DeserializationFailed(msg) => write!(f, "Deserialization failed: {}", msg),
        }
    }
}

impl Error for FactorySpawnError {}

// Args for the init function of the spawned canister
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InitArgs {
    pub owner: String,             // ICP Principal String
    pub nickname: Option<String>,  // Optional nickname
    pub note: Option<String>,      // Optional note
    pub spawn_redeem_code: Option<String>, // Optional redeem code
}

// HTTP Response struct for parsing the redeem endpoint response
#[derive(Debug, Deserialize)]
struct HttpResponse {
    status: u16,
    body: Vec<u8>,
}

// API Response wrapper for the redeem endpoint
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: u32,
    message: String,
}

pub fn is_local_environment() -> bool {
    return LOCAL_DEV_MODE;
    // // Approach 1: Check for an environment variable at build time
    // #[cfg(feature = "local")]
    // {
    //     return true;
    // }
    
    // // Approach 2: Use a compile-time environment variable
    // #[cfg(feature = "local_env")]
    // {
    //     return true;
    // }
    
    // // Approach 3: Define a constant in your code that you can change
    // if LOCAL_DEV_MODE {
    //     return true;
    // }
    
    // // If none of the above conditions are met, check if we're running in a test environment
    // let time_nanos = ic_cdk::api::time();
    // // In test/local environment, the time is often set to a specific value or starts from a low number
    // if time_nanos < 1_000_000_000_000_000_000 { // If time is before ~2001 (very rough estimate)
    //     return true;
    // }
    
    // // If we can't determine for sure, assume production
    // false
}

pub fn get_appropriate_url_endpoint() -> String {
    if is_local_environment() {
        // For local development, use the correct local format with canister ID
        let drive_id = ic_cdk::api::id().to_text();
        
        // Use the configured local port if available
        let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
        
        // In local development, URLs are typically structured like:
        // http://{drive_id}.localhost:{port}
        format!("http://{}.localhost:{}", drive_id, port)
    } else {
        // In production, use the standard IC URL format
        format!("https://{}.icp0.io", ic_cdk::api::id().to_text())
    }
}
