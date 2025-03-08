// src/core/api/factory.rs

use crate::{core::{state::drives::state::state::{DRIVE_ID, RECENT_DEPLOYMENTS, URL_ENDPOINT}, types::UserID}, debug_log, rest::organization::types::{FactorySpawnOrgResponseData, RedeemSpawnOrgRequestBody, RedeemSpawnOrgResponseData}, LOCAL_DEV_MODE};
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

// Constants
const WASM_MODULE: &[u8] = include_bytes!(
    "../../../../../target/wasm32-unknown-unknown/release/canisters_official_backend.wasm"
);

pub async fn factory_spawn_organization_drive(
    owner_id: UserID,
    nickname: Option<String>,
) -> Result<FactorySpawnOrgResponseData, FactorySpawnError> {
    debug_log!("Starting factory spawn organization drive for owner: {}", owner_id);

    // Extract ICP principal from UserID
    // Assuming UserID format is "UserID_<principal>"
    let owner_principal = if owner_id.0.starts_with("UserID_") {
        owner_id.0[7..].to_string()
    } else {
        return Err(FactorySpawnError::CanisterCreationFailed(
            "Invalid UserID format".to_string(),
        ));
    };

    // Validate the ICP principal
    match crate::rest::types::validate_icp_principal(&owner_principal) {
        Ok(_) => {},
        Err(validation_error) => {
            return Err(FactorySpawnError::CanisterCreationFailed(
                format!("Invalid ICP principal: {}", validation_error.message),
            ));
        }
    }

    // Generate redeem code using timestamp
    let timestamp_ns = ic_cdk::api::time();
    let timestamp_ms = timestamp_ns / 1_000_000;
    let spawn_redeem_code = timestamp_ns.to_string();

    // Get factory information
    let factory_drive_id = DRIVE_ID.with(|id| id.clone());
    let factory_url_endpoint = URL_ENDPOINT.with(|url| url.borrow().clone());
    
    let note = format!(
        "Factory spawned from \"{}\" at endpoint \"{}\", at timestamp ms {}",
        factory_drive_id.0, factory_url_endpoint.0, timestamp_ms
    );

    debug_log!("Creating new canister with cycle amount: 1_000_000_000_000");

    // Step 1: Create a new canister
    let create_canister_arg = CreateCanisterArgument {
        settings: Some(ic_cdk::api::management_canister::main::CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            // Added fields that were missing in your errors
            log_visibility: None,
            wasm_memory_limit: None,
        }),
    };

    let cycles_to_use = 1_000_000_000_000u128; // 1T cycles

    let create_result = create_canister(create_canister_arg, cycles_to_use).await;
    
    let canister_id = match create_result {
        Ok((record,)) => record.canister_id,
        Err((code, msg)) => {
            return Err(FactorySpawnError::CanisterCreationFailed(format!(
                "Failed to create canister: code {:?}, message: {}",
                code, msg
            )));
        }
    };

    debug_log!("Created new canister with ID: {}", canister_id);

    // Create the init arguments for the new canister
    let init_args = InitArgs {
        owner: owner_principal,
        nickname: nickname.clone(),
        note: Some(note.clone()),
        spawn_redeem_code: Some(spawn_redeem_code.clone()),
    };

    let encoded_args = Encode!(&init_args).unwrap_or_default();

    // Install the code in the new canister
    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: WASM_MODULE.to_vec(),
        arg: encoded_args,
    };

    debug_log!("Installing code with mode: {:?}", install_code_arg.mode);

    let install_result = install_code(install_code_arg).await;

    match install_result {
        Ok(()) => debug_log!("Successfully installed code in canister: {}", canister_id),
        Err((code, msg)) => {
            return Err(FactorySpawnError::InstallCodeFailed(format!(
                "Failed to install code: code {:?}, message: {}",
                code, msg
            )));
        }
    }

    // Use the factory_url_endpoint directly
    let spawn_url_endpoint = factory_url_endpoint.0.clone();
    
    // Format the redeem endpoint
    let redeem_endpoint = format!(
        "/v1/DriveID_{}/organization/redeem_spawn", 
        canister_id.to_text()
    );

    // Create the request body
    let request_body = RedeemSpawnOrgRequestBody {
        redeem_code: spawn_redeem_code,
    };

    let request_body_json = serde_json::to_string(&request_body).unwrap_or_default();

    let request = CanisterHttpRequestArgument {
        url: format!("{}{}", spawn_url_endpoint, redeem_endpoint),
        method: HttpMethod::POST,
        body: Some(request_body_json.into_bytes()),
        max_response_bytes: Some(2 * 1024 * 1024), // 2MB limit
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                method: "redeem_spawn".to_string(),
                principal: ic_cdk::api::id(),
            }),
            context: vec![],
        }),
        // Fix HttpHeader format
        headers: vec![HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }],
    };

    debug_log!("Sending HTTP request to: {}{}", spawn_url_endpoint, redeem_endpoint);

    let http_result = http_request(request, 50_000_000).await;

    let response = match http_result {
        Ok((response,)) => response,
        Err((code, msg)) => {
            return Err(FactorySpawnError::HttpCallFailed(format!(
                "HTTP request failed: code {:?}, message: {}",
                code, msg
            )));
        }
    };

    // Parse the response
    let response_body = String::from_utf8(response.body)
        .map_err(|e| {
            FactorySpawnError::DeserializationFailed(format!(
                "Failed to parse response body as UTF-8: {}",
                e
            ))
        })?;

    let api_response: ApiResponse<RedeemSpawnOrgResponseData> =
        serde_json::from_str(&response_body).map_err(|e| {
            FactorySpawnError::DeserializationFailed(format!(
                "Failed to deserialize API response: {}",
                e
            ))
        })?;

    if !api_response.success {
        return Err(FactorySpawnError::HttpCallFailed(format!(
            "API call failed: {}",
            api_response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    let redeem_data = api_response.data.ok_or_else(|| {
        FactorySpawnError::DeserializationFailed("Missing data in API response".to_string())
    })?;

    // Record the deployment in history
    RECENT_DEPLOYMENTS.with(|deployments| {
        let record = FactorySpawnHistoryRecord {
            owner_id: owner_id.clone(),
            drive_id: redeem_data.drive_id.clone(),
            endpoint: redeem_data.endpoint.clone(),
        };
        deployments.borrow_mut().push(record);
    });

    // Create the response with the data from the redeem_spawn endpoint
    let response_data = FactorySpawnOrgResponseData {
        drive_id: redeem_data.drive_id,
        endpoint: redeem_data.endpoint,
        api_key: redeem_data.api_key,
        note: redeem_data.note,
        admin_login_password: redeem_data.admin_login_password,
    };

    debug_log!("Successfully spawned organization drive: {}", response_data.drive_id);
    Ok(response_data)
}

pub fn is_local_environment() -> bool {
    // Approach 1: Check for an environment variable at build time
    #[cfg(feature = "local")]
    {
        return true;
    }
    
    // Approach 2: Use a compile-time environment variable
    #[cfg(feature = "local_env")]
    {
        return true;
    }
    
    // Approach 3: Define a constant in your code that you can change
    if LOCAL_DEV_MODE {
        return true;
    }
    
    // If none of the above conditions are met, check if we're running in a test environment
    let time_nanos = ic_cdk::api::time();
    // In test/local environment, the time is often set to a specific value or starts from a low number
    if time_nanos < 1_000_000_000_000_000_000 { // If time is before ~2001 (very rough estimate)
        return true;
    }
    
    // If we can't determine for sure, assume production
    false
}

pub fn get_appropriate_url_endpoint() -> String {
    if is_local_environment() {
        // For local development, use the correct local format with canister ID
        let canister_id = ic_cdk::api::id().to_text();
        
        // Use the configured local port if available
        let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
        
        // In local development, URLs are typically structured like:
        // http://{canister_id}.localhost:{port}
        format!("http://{}.localhost:{}", canister_id, port)
    } else {
        // In production, use the standard IC URL format
        format!("https://{}.icp0.io", ic_cdk::api::id().to_text())
    }
}
