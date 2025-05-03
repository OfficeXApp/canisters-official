// src/lib.rs
use ic_cdk::*;
use ic_http_certification::{HttpRequest, HttpResponse};
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl, StableCell};
use core::{api::uuid::format_user_id, state::{api_keys::state::state::init_default_admin_apikey, giftcards_spawnorg::state::state::init_self_factory}, types::UserID};
use std::{cell::RefCell, collections::HashMap};
use serde::{Deserialize, Serialize};
use bip39::{Mnemonic, Language};
mod logger;
mod rest;
mod core;
use rest::{router, types::validate_icp_principal};
use candid::{CandidType, Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, VirtualMemory};


// LOCAL_DEV
pub static _DEPLOYMENT_STAGING: DEPLOYMENT_STAGE = DEPLOYMENT_STAGE::StagingPublicTestnet;


// rust enum for stage
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum DEPLOYMENT_STAGE {
    LocalDevelopment,
    StagingPublicTestnet,
    Production
}


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InitArgs {
    pub owner: String, // Plain string for simplicity, really should be ICPPrincipalString
}

type Memory = VirtualMemory<DefaultMemoryImpl>;

const INITIALIZED_FLAG_MEMORY_ID: MemoryId = MemoryId::new(0);

// Track if we've already initialized to prevent double initialization
thread_local! {

    // The memory manager is used for simulating multiple memories.
    pub(crate) static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Stable Cell for the INITIALIZED flag. Uses MemoryId(0).
    // We store a u8: 0 = false, 1 = true, as bool support might vary or have quirks.
    // Alternatively, you could create a custom struct/enum that implements Storable.
    // Using `bool` directly *should* work if it implements Storable (which it does via candid).
    // Let's try with bool first for clarity, but u8 is a safe fallback.
    static INITIALIZED_FLAG: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(INITIALIZED_FLAG_MEMORY_ID)),
            false // Default value if the cell is newly created (e.g., first deployment)
        ).expect("Failed to initialize StableCell for INITIALIZED_FLAG")
    );

    // --- Other State (potentially also using stable structures) ---
    // Example: If you were to move other state to stable structures
    // static CONTACTS: RefCell<StableBTreeMap<UserID, Contact, Memory>> = RefCell::new(
    //     StableBTreeMap::init(
    //         MEMORY_MANAGER.with(|m| m.borrow().get(CONTACTS_MEMORY_ID)),
    //     )
    // );
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
    debug_log!("INIT FUNCTION - Router initialized");
    router::init_routes();

    // Check if we've already initialized to prevent re-initialization
    let already_initialized = INITIALIZED_FLAG.with(|flag_cell| {
        *flag_cell.borrow().get() // Get the value from the stable cell
    });

    if already_initialized {
        debug_log!("Canister already initialized, skipping initialization");
        return;
    }

    debug_log!("Initializing canister...");
    
    // Process the arguments
    if let Some(init_args) = args {
        // Validate the owner ICP principal
        match validate_icp_principal(&init_args.owner) {
            Ok(_) => {
                // Convert ICP principal to UserID format
                let owner_id = format_user_id(&init_args.owner);


                // Initialize state stable structures
                crate::core::state::api_keys::state::state::initialize();
                crate::core::state::giftcards_spawnorg::state::state::initialize();
                crate::core::state::giftcards_refuel::state::state::initialize();
                
                // Initialize the drive with all parameters
                init_self_factory(
                    owner_id
                );

                // Verify the values were set correctly
                crate::core::state::giftcards_spawnorg::state::state::OWNER_ID.with(|id| {
                    debug_log!("After init, owner_id is: {}", id.borrow().get().0);
                });
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
    
    init_default_admin_apikey();

    // Set the initialized flag to true
    INITIALIZED_FLAG.with(|flag_cell| {
        flag_cell.borrow_mut().set(true);
    });
}

#[post_upgrade]
fn post_upgrade() {
    // No arguments on upgrade, just re-initialize routes
    debug_log!("Post-upgrade initialization...");

    router::init_routes();

    // Then check if we need to set up state
    let already_initialized = INITIALIZED_FLAG.with(|flag_cell| {
        *flag_cell.borrow().get() // Get the value from the stable cell
    });
    
    if already_initialized {
        debug_log!("Canister already initialized, skipping full initialization");
    } else {
         // Either use arguments from upgrade call or fallback to defaults
         let args = ic_cdk::api::call::arg_data::<(Option<InitArgs>,)>(ic_cdk::api::call::ArgDecoderConfig::default()).0;
         initialize_canister(args);
    }
}

#[query]
fn http_request(_req: HttpRequest) -> HttpResponse<'static> {
    // All requests will be upgraded to update calls
    HttpResponse::builder()
        .with_upgrade(true)
        .build()
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'static> {
    router::handle_request(req).await
}
