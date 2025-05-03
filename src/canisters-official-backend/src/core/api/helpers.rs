// src/core/api/factory.rs

use crate::{core::{state::drives::state::state::{DRIVE_ID, RECENT_DEPLOYMENTS, URL_ENDPOINT}, types::UserID}, debug_log, rest::organization::types::{ RedeemOrgRequestBody, RedeemOrgResponseData}, DEPLOYMENT_STAGE, _DEPLOYMENT_STAGING};
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


pub fn get_appropriate_url_endpoint() -> String {
    if _DEPLOYMENT_STAGING == DEPLOYMENT_STAGE::Production {
// In production, use the standard IC URL format
format!("https://{}.icp0.io", ic_cdk::api::id().to_text())
    } else if _DEPLOYMENT_STAGING == DEPLOYMENT_STAGE::StagingPublicTestnet {
        format!("https://{}.icp-testnet.click", ic_cdk::api::id().to_text())
    } else {
        // For local development, use the correct local format with canister ID
        let drive_id = ic_cdk::api::id().to_text();
        
        // Use the configured local port if available
        let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
        
        // In local development, URLs are typically structured like:
        // http://{drive_id}.localhost:{port}
        format!("http://{}.localhost:{}", drive_id, port)
    }
}
