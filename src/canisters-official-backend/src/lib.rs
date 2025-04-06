// src/lib.rs
use ic_cdk::*;
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use core::{api::uuid::format_user_id, state::{api_keys::state::state::init_default_admin_apikey, contacts::state::state::init_default_owner_contact, disks::state::state::init_default_disks, drives::state::state::init_self_drive}, types::UserID};
use std::{cell::RefCell, collections::HashMap};
use serde::{Deserialize, Serialize};
use bip39::{Mnemonic, Language};
mod logger;
mod rest;
mod core;
use rest::{router, types::validate_icp_principal};
use candid::{CandidType, Decode, Encode};


// change this to false for production
pub static LOCAL_DEV_MODE: bool = true;


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InitArgs {
    pub owner: String, // Plain string for simplicity, really should be ICPPrincipalString
    pub title: Option<String>,
    pub owner_name: Option<String>,
    pub note: Option<String>,
    pub spawn_redeem_code: Option<String>,
}

// Track if we've already initialized to prevent double initialization
thread_local! {
    static INITIALIZED: RefCell<bool> = RefCell::new(false);
}


#[ic_cdk_macros::init]
fn init() {
    debug_log!("INIT FUNCTION STARTED - Extracting arguments...");
    let args = ic_cdk::api::call::arg_data::<(Option<InitArgs>,)>(ic_cdk::api::call::ArgDecoderConfig::default()).0;
    debug_log!("INIT FUNCTION - Args extracted, calling initialize_canister...");
    initialize_canister(args);
    debug_log!("INIT FUNCTION COMPLETED");
}


fn initialize_canister(args: Option<InitArgs>) {
    // Check if we've already initialized to prevent re-initialization
    let already_initialized = INITIALIZED.with(|initialized| {
        if *initialized.borrow() {
            true
        } else {
            *initialized.borrow_mut() = true;
            false
        }
    });

    if already_initialized {
        debug_log!("Canister already initialized, skipping initialization");
        return;
    }

    debug_log!("Initializing canister...");
    router::init_routes();
    
    // Process the arguments
    if let Some(init_args) = args {
        // Validate the owner ICP principal
        match validate_icp_principal(&init_args.owner) {
            Ok(_) => {
                // Convert ICP principal to UserID format
                let owner_id = format_user_id(&init_args.owner);
                
                // Initialize the drive with all parameters
                init_self_drive(
                    owner_id,
                    init_args.title,
                    init_args.spawn_redeem_code,
                    init_args.note,
                );

                // Verify the values were set correctly
                crate::core::state::drives::state::state::OWNER_ID.with(|id| {
                    debug_log!("After init, owner_id is: {}", id.borrow().0);
                });
                
                crate::core::state::drives::state::state::SPAWN_REDEEM_CODE.with(|code| {
                    debug_log!("After init, spawn_redeem_code is: {}", code.borrow().0);
                });

                init_default_admin_apikey();
                init_default_owner_contact(init_args.owner_name);
                init_default_disks();
            },
            Err(validation_error) => {
                // Log and trap (abort) on invalid ICP principal
                debug_log!("FATAL: Invalid owner ICP principal: {}", validation_error.message);
                ic_cdk::trap(&format!("Initialization failed: Invalid owner ICP principal - {}", 
                    validation_error.message));
            }
        }
    } else {
        // Fail initialization if no arguments are provided
        debug_log!("FATAL: No initialization arguments provided");
        ic_cdk::trap("Initialization failed: Owner principal is required");
    }
    
}

#[post_upgrade]
fn post_upgrade() {
    // No arguments on upgrade, just re-initialize routes
    debug_log!("Post-upgrade initialization...");
    // Then check if we need to set up state
    let needs_init = INITIALIZED.with(|initialized| !*initialized.borrow());
    
    if needs_init {
        // Either use arguments from upgrade call or fallback to defaults
        let args = ic_cdk::api::call::arg_data::<(Option<InitArgs>,)>(ic_cdk::api::call::ArgDecoderConfig::default()).0;
        initialize_canister(args);
    } else {
        debug_log!("Canister already initialized, skipping full initialization");
    }
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse<'static> {

    // All requests will be upgraded to update calls
    HttpResponse::builder()
        .with_upgrade(true)
        .build()
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'static> {
    router::handle_request(req).await
}
