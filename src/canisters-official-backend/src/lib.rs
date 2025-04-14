// src/lib.rs
use ic_cdk::{api::stable::{StableReader, StableWriter}, *};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use core::{api::{replay::diff::{apply_entire_state, snapshot_entire_state, EntireState}, uuid::format_user_id}, state::{api_keys::state::state::init_default_admin_apikey, contacts::state::state::init_default_owner_contact, disks::state::state::init_default_disks, drives::state::state::init_self_drive, groups::state::state::init_default_group}, types::UserID};
use std::{borrow::Cow, cell::{RefCell}, collections::HashMap};
use serde::{Deserialize, Serialize};
use bip39::{Mnemonic, Language};
mod logger;
mod rest;
mod core;
use rest::{router, types::validate_icp_principal};
use candid::{encode_one, CandidType, Decode, Encode};
use ic_stable_structures::{Cell, memory_manager::{MemoryId, MemoryManager, VirtualMemory}, DefaultMemoryImpl, Storable};



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


// Define the types we need
type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

// Define stable memory cells for state management
thread_local! {
    // Memory manager for stable storage
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = 
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    
    // Cell for tracking if canister is initialized
    static INITIALIZED: RefCell<Cell<bool, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            false
        ).unwrap()
    );
    
    // Cell for storing the serialized state size
    static STATE_SIZE: RefCell<Cell<u64, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            0
        ).unwrap()
    );
    
    // We'll use a region of memory starting at ID 2 for the actual state data
    static STATE_MEMORY: RefCell<Memory> = RefCell::new(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    );
}

/// Query function to check if the canister has been initialized
#[query]
fn is_initialized() -> bool {
    let _is_initialized = INITIALIZED.with(|cell| cell.borrow().get().clone());
    debug_log!("is_initialized: {}", _is_initialized);
    _is_initialized
}

/// Update function to set the initialized flag to true
#[update]
fn set_initialized() -> bool {
    INITIALIZED.with(|cell| {
        let was_initialized = cell.borrow().get().clone();
        cell.borrow_mut().set(true).unwrap();
        was_initialized
    })
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
    if is_initialized() {
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
                init_default_group();

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

#[pre_upgrade]
fn pre_upgrade() {
    debug_log!("Starting pre_upgrade - preparing for canister upgrade");
    
    // Take a snapshot of the entire state
    let state_snapshot = snapshot_entire_state();
    debug_log!("State snapshot taken");
    
    // Use direct binary serialization with Candid
    let encoded_state = match encode_one(state_snapshot) {
        Ok(bytes) => bytes,
        Err(e) => {
            debug_log!("FATAL: Failed to serialize state for upgrade: {}", e);
            ic_cdk::trap(&format!("Failed to serialize state for upgrade: {}", e));
        }
    };
    
    let state_size = encoded_state.len() as u64;
    debug_log!("State serialized, size: {} bytes", state_size);
    
    // Write to stable storage
    let mut writer = StableWriter::default();
    
    // First, write the size of the serialized data as 8 bytes
    match writer.write(&state_size.to_be_bytes()) {
        Ok(_) => debug_log!("Successfully wrote state size to stable storage"),
        Err(e) => {
            debug_log!("FATAL: Failed to write state size to stable storage: {}", e);
            ic_cdk::trap(&format!("Failed to write state size to stable storage: {}", e));
        }
    }
    
    // Then, write the actual serialized data
    match writer.write(&encoded_state) {
        Ok(_) => debug_log!("Successfully wrote state data to stable storage"),
        Err(e) => {
            debug_log!("FATAL: Failed to write state data to stable storage: {}", e);
            ic_cdk::trap(&format!("Failed to write state data to stable storage: {}", e));
        }
    }
    
    debug_log!("Pre-upgrade serialization completed successfully");
}

#[post_upgrade]
fn post_upgrade() {
    debug_log!("Starting post_upgrade - restoring from upgrade");
    
    // Create a reader for the stable storage
    let mut reader = StableReader::default();
    
    // First, read the size of the serialized data
    let mut size_buffer = [0u8; 8]; // 8 bytes for u64
    
    match reader.read(&mut size_buffer) {
        Ok(bytes_read) if bytes_read == 8 => {
            // Convert bytes to u64 (size of our serialized data)
            let size = u64::from_be_bytes(size_buffer);
            debug_log!("Found serialized state of size: {} bytes", size);
            
            if size > 0 && size < 4_000_000_000 { // Sanity check on size
                // Create a buffer of the exact size needed
                let mut state_buffer = vec![0u8; size as usize];
                
                // Read the actual state data
                match reader.read(&mut state_buffer) {
                    Ok(bytes_read) if bytes_read as u64 == size => {
                        debug_log!("Successfully read {} bytes of state data", bytes_read);
                        
                        // Deserialize the state using Candid
                        match Decode!(&state_buffer, EntireState) {
                            Ok(state) => {
                                debug_log!("Successfully deserialized state, restoring...");
                                
                                // IMPORTANT: Apply the state WITHOUT initializing routes
                                // This preserves all existing route registrations
                                apply_entire_state(state);
                                
                                // Mark as initialized after successful restoration
                                set_initialized();
                                
                                debug_log!("State successfully restored from upgrade");
                                return; // Exit early on successful restore
                            },
                            Err(e) => {
                                debug_log!("Error deserializing state from upgrade: {:?}", e);
                                // Fall through to fresh initialization
                            }
                        }
                    },
                    Ok(bytes_read) => {
                        debug_log!("Partial read of state data: expected {} bytes, got {}", 
                                   size, bytes_read);
                        // Fall through to fresh initialization
                    },
                    Err(e) => {
                        debug_log!("Error reading state data from stable storage: {:?}", e);
                        // Fall through to fresh initialization
                    }
                }
            } else {
                debug_log!("Invalid state size: {}", size);
                // Fall through to fresh initialization
            }
        },
        Ok(_) => {
            debug_log!("Failed to read state size header");
            // Fall through to fresh initialization
        },
        Err(e) => {
            debug_log!("Error reading state size from stable storage: {:?}", e);
            // Fall through to fresh initialization
        }
    }
    
    // Only reach here if state restoration failed
    debug_log!("State restoration failed, falling back to fresh initialization");
    
    // For fresh initialization, we DO need to initialize routes
    router::init_routes();
    
    let args = ic_cdk::api::call::arg_data::<(Option<InitArgs>,)>(
        ic_cdk::api::call::ArgDecoderConfig::default()
    ).0;
    initialize_canister(args);
    
    debug_log!("Post-upgrade completed");
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
