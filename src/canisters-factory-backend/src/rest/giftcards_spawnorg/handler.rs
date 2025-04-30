// src/rest/giftcards_spawnorg/handler.rs

pub mod giftcards_handlers {
    use std::{thread::sleep, time::Duration};
    use crate::core::api::helpers::is_local_environment;
    use crate::core::api::uuid::format_drive_id;
    use crate::core::state::giftcards_spawnorg::types::DriveID;
    use crate::core::state::giftcards_spawnorg::types::DriveRESTUrlEndpoint;
    use crate::core::state::giftcards_spawnorg::types::FactorySpawnHistoryRecord;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrgID;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrg;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrgIDVec;
    use crate::rest::giftcards_spawnorg::types::RedeemGiftcardSpawnOrgData;
    use crate::rest::giftcards_spawnorg::types::RedeemGiftcardSpawnOrgResult;
    use crate::rest::giftcards_spawnorg::types::SpawnInitArgs;
    use crate::{
        core::{
            api::uuid::{format_user_id, generate_uuidv4}, 
            state::giftcards_spawnorg::{
                    state::state::{HISTORICAL_GIFTCARDS_SPAWNORGS, OWNER_ID, USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE, GIFTCARD_SPAWNORG_BY_ID},
                    
                }, 
            types::{IDPrefix, UserID}
        }, 
        debug_log, 
        rest::{
            auth::{authenticate_request, create_auth_error_response}, giftcards_spawnorg::types::{
                CreateGiftcardSpawnOrgRequestBody, CreateGiftcardSpawnOrgResponse, DeleteGiftcardSpawnOrgRequestBody, DeleteGiftcardSpawnOrgResponse, DeletedGiftcardSpawnOrgData, ErrorResponse, GetGiftcardSpawnOrgResponse, ListGiftcardSpawnOrgsRequestBody, ListGiftcardSpawnOrgsResponse, ListGiftcardSpawnOrgsResponseData, RedeemGiftcardSpawnOrgResponse, SortDirection, UpdateGiftcardSpawnOrgRequestBody, UpdateGiftcardSpawnOrgResponse, UpsertGiftcardSpawnOrgRequestBody
            }
        }, 
    };
    use candid::Nat;
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        note: Option<String>,
    }

    pub async fn get_giftcard_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Get the requested giftcard ID from params
        let requested_id = GiftcardSpawnOrgID(params.get("giftcard_id").unwrap().to_string());

        // Get the requested giftcard
        let giftcard = GIFTCARD_SPAWNORG_BY_ID.with(|store| {
            store.borrow().get(&requested_id).clone()
        });

        // Check authorization (only owner can view giftcards)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        if !is_owner {
            return create_auth_error_response();
        }
 
        match giftcard {
            Some(v) => {
                create_response(
                    StatusCode::OK,
                    GetGiftcardSpawnOrgResponse::ok(&v).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "GiftcardSpawnOrg not found".to_string()).encode()
            ),
        }
    }

    pub async fn list_giftcards_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming list giftcards request: {}", request.url());

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check authorization - only owner can list all giftcards
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body = request.body();
        let request_body: ListGiftcardSpawnOrgsRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        // Validate request body
        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        // Parse cursors if provided
        let start_cursor = if let Some(cursor) = request_body.cursor {
            match cursor.parse::<usize>() {
                Ok(idx) => Some(idx),
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid cursor format".to_string()).encode()
                ),
            }
        } else {
            None
        };


        // Get total count from historical giftcards
        let total_count = HISTORICAL_GIFTCARDS_SPAWNORGS.with(|historical_ids| {
            historical_ids.borrow().len()
        });

        // If there are no giftcards, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListGiftcardSpawnOrgsResponse::ok(&ListGiftcardSpawnOrgsResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    direction: request_body.direction,
                    cursor: None,
                }).encode()
            );
        }

        // Determine starting point based on cursors
        let start_index = if let Some(cursor_idx) = start_cursor {
            cursor_idx.min(total_count as usize - 1)
        } else {
            match request_body.direction {
                SortDirection::Asc => 0,
                SortDirection::Desc => total_count as usize - 1,
            }
        };
    

        // Get giftcards with pagination and filtering
        let mut filtered_giftcards = Vec::new();
        let mut processed_count = 0;
        let mut end_index = start_index;

        HISTORICAL_GIFTCARDS_SPAWNORGS.with(|historical_ids| {
            let historical_ids = historical_ids.borrow();
            GIFTCARD_SPAWNORG_BY_ID.with(|store| {
                let store = store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_giftcards.len() < request_body.page_size && current_idx < total_count as usize {
                            // Use .get() method instead of [] indexing
                            if let Some(id) = historical_ids.get(current_idx as u64) {
                                if let Some(giftcard) = store.get(&id) {
                                    // Apply filters if any
                                    if request_body.filters.is_empty() || 
                                       (giftcard.note.to_lowercase().contains(&request_body.filters.to_lowercase())) {
                                        filtered_giftcards.push(giftcard.clone());
                                    }
                                }
                            }
                            if current_idx == 0 {
                                break;
                            }
                            current_idx -= 1;
                            processed_count = start_index - current_idx;
                        }
                    },
                    SortDirection::Asc => {
                        let mut current_idx = start_index;
                        while filtered_giftcards.len() < request_body.page_size && current_idx < total_count as usize {
                            // Use .get() method instead of [] indexing
                            if let Some(id) = historical_ids.get(current_idx as u64) {
                                if let Some(giftcard) = store.get(&id) {
                                    // Apply filters if any
                                    if request_body.filters.is_empty() || 
                                       (giftcard.note.to_lowercase().contains(&request_body.filters.to_lowercase())) {
                                        filtered_giftcards.push(giftcard.clone());
                                    }
                                }
                            }
                            current_idx += 1;
                            processed_count = current_idx - start_index;
                            if current_idx >= total_count as usize {
                                break;
                            }
                        }
                    }
                }
            });
        });

        // Calculate next cursors based on direction and current position
        let next_cursor = if filtered_giftcards.len() >= request_body.page_size {
            match request_body.direction {
                SortDirection::Desc => {
                    if end_index > 0 {
                        Some(end_index.to_string())
                    } else {
                        None
                    }
                },
                SortDirection::Asc => {
                    if end_index < total_count as usize - 1 {
                        Some((end_index + 1).to_string())
                    } else {
                        None
                    }
                }
            }
        } else {
            None  // No more results available
        };

        create_response(
            StatusCode::OK,
            ListGiftcardSpawnOrgsResponse::ok(&ListGiftcardSpawnOrgsResponseData {
                items: filtered_giftcards.clone(),
                page_size: filtered_giftcards.len(),
                total: total_count as usize,
                direction: request_body.direction,
                cursor: next_cursor,
            }).encode()
        )
    }

    pub async fn upsert_giftcard_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check if requester is owner (only owner can create/update giftcards)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        if !is_owner {
            return create_auth_error_response();
        }
    
        // Parse request body
        let body: &[u8] = request.body();

        if let Ok(req) = serde_json::from_slice::<UpsertGiftcardSpawnOrgRequestBody>(body) {
            // Validate request body
            if let Err(validation_error) = req.validate_body() {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(
                        400,
                        format!("Validation error for field '{}': {}", validation_error.field, validation_error.message)
                    ).encode()
                );
            }

            match req {
                UpsertGiftcardSpawnOrgRequestBody::Create(create_req) => {            
                    // Create new giftcard
                    let current_time = ic_cdk::api::time() / 1_000_000;
                    let new_giftcard = GiftcardSpawnOrg {
                        id: GiftcardSpawnOrgID(generate_uuidv4(IDPrefix::GiftcardSpawnOrg)),
                        usd_revenue_cents: create_req.usd_revenue_cents,
                        note: create_req.note,
                        gas_cycles_included: create_req.gas_cycles_included,
                        timestamp_ms: current_time,
                        external_id: create_req.external_id,
                        redeemed: false,
                        disk_auth_json: create_req.disk_auth_json,
                    };
            
                    // Add to GIFTCARD_SPAWNORG_BY_ID
                    GIFTCARD_SPAWNORG_BY_ID.with(|store| {
                        store.borrow_mut().insert(new_giftcard.id.clone(), new_giftcard.clone());
                    });
            
                    // Add to USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE for the owner
                    let owner_id = OWNER_ID.with(|id| (*id.borrow().get()).clone());
                    USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE.with(|store| {
                        let mut store_ref = store.borrow_mut();
                        let mut id_vec = match store_ref.get(&owner_id) {
                            Some(v) => v.clone(),
                            None => GiftcardSpawnOrgIDVec::new(),
                        };
                        id_vec.push(new_giftcard.id.clone());
                        store_ref.insert(owner_id, id_vec);
                    });
                    
                    // Add to HISTORICAL_GIFTCARDS_SPAWNORGS
                    crate::core::state::giftcards_spawnorg::state::state::HISTORICAL_GIFTCARDS_SPAWNORGS.with(|giftcards| {
                        giftcards.borrow_mut().push(&new_giftcard.id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        CreateGiftcardSpawnOrgResponse::ok(&new_giftcard).encode()
                    )  
                },
                UpsertGiftcardSpawnOrgRequestBody::Update(update_req) => {
                    // Get the giftcard to update
                    let giftcard_id = GiftcardSpawnOrgID(update_req.id);
                    let mut giftcard = match GIFTCARD_SPAWNORG_BY_ID.with(|store| store.borrow().get(&giftcard_id).clone()) {
                        Some(v) => v,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::err(404, "GiftcardSpawnOrg not found".to_string()).encode()
                        ),
                    };

                    // Update only the fields that were provided
                    if let Some(notes) = update_req.notes {
                        giftcard.note = notes;
                    }
                    if let Some(usd_revenue_cents) = update_req.usd_revenue_cents {
                        giftcard.usd_revenue_cents = usd_revenue_cents;
                    }
                    if let Some(gas_cycles_included) = update_req.gas_cycles_included {
                        giftcard.gas_cycles_included = gas_cycles_included;
                    }
                    if let Some(external_id) = update_req.external_id {
                        giftcard.external_id = external_id;
                    }
                    if let Some(disk_auth_json) = update_req.disk_auth_json {
                        giftcard.disk_auth_json = Some(disk_auth_json);
                    }
            
                    // Update the giftcard in GIFTCARD_SPAWNORG_BY_ID
                    GIFTCARD_SPAWNORG_BY_ID.with(|store| {
                        store.borrow_mut().insert(giftcard.id.clone(), giftcard.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateGiftcardSpawnOrgResponse::ok(&giftcard).encode()
                    )
                }
            }
        } else {
            create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            )
        }
    }

    pub async fn delete_giftcard_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming request: {}", request.url());

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check if requester is owner (only owner can delete giftcards)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body: &[u8] = request.body();
        
        debug_log!("Incoming request body: {}", String::from_utf8_lossy(request.body()));
        let delete_request = match serde_json::from_slice::<DeleteGiftcardSpawnOrgRequestBody>(body) {
            Ok(req) => req,
            Err(_) => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid request format".to_string()).encode()
                )
            }
        };

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400,
                    format!("Validation error for field '{}': {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        // Get the giftcard to be deleted
        let giftcard_id = GiftcardSpawnOrgID(delete_request.id.clone());
        let _giftcard = match GIFTCARD_SPAWNORG_BY_ID.with(|store| store.borrow().get(&giftcard_id).clone()) {
            Some(v) => v,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "GiftcardSpawnOrg not found".to_string()).encode()
                )
            }
        };

        // Remove from GIFTCARD_SPAWNORG_BY_ID
        GIFTCARD_SPAWNORG_BY_ID.with(|store| {
            store.borrow_mut().remove(&giftcard_id);
        });

        // Remove from USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE
        let owner_id = OWNER_ID.with(|id| id.borrow().get().clone());
        USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(giftcard_ids) = store.get(&owner_id) {
                let mut updated_ids = giftcard_ids.clone();
                updated_ids.items.retain(|id| id != &giftcard_id);
                
                // If this was the last giftcard for the user, remove the user entry
                if updated_ids.items.is_empty() {
                    store.remove(&owner_id);
                } else {
                    store.insert(owner_id.clone(), updated_ids);
                }
            }
        });
        
        // Note: We intentionally do NOT remove from HISTORICAL_GIFTCARDS_SPAWNORGS
        // This preserves the historical record even if a giftcard is deleted

        // Return success response
        create_response(
            StatusCode::OK,
            DeleteGiftcardSpawnOrgResponse::ok(&DeletedGiftcardSpawnOrgData {
                id: delete_request.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn redeem_giftcard_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming redeem giftcard request: {}", request.url());
    
        // // Authenticate request
        // let requester_api_key = match authenticate_request(request) {
        //     Some(key) => key,
        //     None => return create_auth_error_response(),
        // };
    
        // Parse request body
        let body: &[u8] = request.body();
        
        let redeem_request = match serde_json::from_slice::<RedeemGiftcardSpawnOrgData>(body) {
            Ok(req) => req,
            Err(_) => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid request format".to_string()).encode()
                )
            }
        };
    
        // Validate request body
        if let Err(validation_error) = redeem_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }
    
        // Get the giftcard to be redeemed
        let giftcard_id = redeem_request.giftcard_id.clone();
        let giftcard = match GIFTCARD_SPAWNORG_BY_ID.with(|store| store.borrow().get(&giftcard_id).clone()) {
            Some(v) => v,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "GiftcardSpawnOrg not found".to_string()).encode()
                )
            }
        };
    
        // Check if giftcard is already redeemed
        if giftcard.redeemed {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "GiftcardSpawnOrg already redeemed".to_string()).encode()
            );
        }
    
        // Generate a unique redeem code using timestamp
        let redeem_code = format!("REDEEM_{}", ic_cdk::api::time());
    
        // Convert ICP principal to UserID format
        let owner_id = format_user_id(&redeem_request.owner_icp_principal);
    
        // Create note for the factory spawn
        let current_time = ic_cdk::api::time() / 1_000_000;
        let time_iso = format_iso8601(current_time);
        
        // Deploy the canister using IC management canister
        let deployed_canister = match deploy_drive_canister(
            redeem_request.owner_icp_principal.clone(),
            redeem_request.organization_name.clone(),
            redeem_request.owner_name.clone(),
            redeem_code.clone(),
            Some(format!("giftcard {} was redeemed to spawn drive with {} cycles, owned by {}, on timestamp_ms {} {}", 
                giftcard_id.0, giftcard.gas_cycles_included, owner_id.0, current_time, time_iso)),
            giftcard.gas_cycles_included,
        ).await {
            Ok(canister_id) => canister_id,
            Err(e) => {
                return create_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::err(500, format!("Failed to deploy canister: {}", e)).encode()
                )
            }
        };
    
        // Create a record of the deployment
        let version = crate::core::state::giftcards_spawnorg::state::state::VERSION.with(|v| v.borrow().get().clone());
        
        // Get appropriate URL endpoint for the deployed canister
        let endpoint = match is_local_environment() {
            true => {
                // Use the configured local port if available
                let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
                
                // In local development, URLs are typically structured like:
                // http://{canister_id}.localhost:{port}
                DriveRESTUrlEndpoint(format!("http://{}.localhost:{}", deployed_canister, port))
            },
            false => {
                // In production, use the standard IC URL format
                DriveRESTUrlEndpoint(format!("https://{}.icp0.io", deployed_canister))
            }
        };
        
        let history_record = FactorySpawnHistoryRecord {
            owner_id: owner_id.clone(),
            drive_id: format_drive_id(&deployed_canister.clone()),
            endpoint: endpoint.clone(),
            version,
            note: giftcard.note.clone(),
            gas_cycles_included: giftcard.gas_cycles_included,
            timestamp_ms: current_time,
            giftcard_id: giftcard_id.clone(),
        };
    
        // Update giftcard as redeemed
        GIFTCARD_SPAWNORG_BY_ID.with(|store| {
            let mut giftcard = giftcard.clone();
            giftcard.redeemed = true;
            store.borrow_mut().insert(giftcard_id.clone(), giftcard);
        });
    
        // Store the deployment history
        crate::core::state::giftcards_spawnorg::state::state::DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID.with(|records| {
            records.borrow_mut().insert(giftcard_id.clone(), history_record.clone());
        });
    
        // Add to DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE
        crate::core::state::giftcards_spawnorg::state::state::DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE.with(|map| {
            map.borrow_mut().insert(format_drive_id(&deployed_canister.clone()), giftcard_id.clone());
        });
    
        
        // Add to USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE for the owner
        USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE.with(|store| {
            let mut store_ref = store.borrow_mut();
            let mut id_vec = match store_ref.get(&owner_id) {
                Some(v) => v.clone(),
                None => GiftcardSpawnOrgIDVec::new(),
            };
            id_vec.push(giftcard_id.clone());
            store_ref.insert(owner_id.clone(), id_vec);
        });

        let redeem_giftcard_result = RedeemGiftcardSpawnOrgResult {
            owner_id: owner_id,
            drive_id: format_drive_id(&deployed_canister),
            endpoint: endpoint,
            redeem_code: redeem_code,
            disk_auth_json: giftcard.disk_auth_json,
        };
    
        // Return success response
        create_response(
            StatusCode::OK,
            RedeemGiftcardSpawnOrgResponse::ok(&redeem_giftcard_result).encode()
        )
    }
    
    
    // Modified deploy_drive_canister function to use chunked code installation
    async fn deploy_drive_canister(
        owner_icp_principal: String, 
        title: Option<String>, 
        owner_name: Option<String>,
        spawn_redeem_code: String,
        note: Option<String>,
        cycles: u64
    ) -> Result<String, String> {
        use ic_cdk::api::management_canister::main::{
            create_canister, install_chunked_code, CanisterInstallMode, CreateCanisterArgument, 
            InstallChunkedCodeArgument, UploadChunkArgument,
        };
        use candid::{Encode, Principal};

        // Convert owner ID to Principal
        let owner_principal = match Principal::from_text(&owner_icp_principal) {
            Ok(p) => p,
            Err(_) => return Err("Invalid owner ICP principal".to_string()),
        };

        debug_log!("Creating drive for owner: {}", owner_icp_principal);

        // temp hardcoded dfx controller principal (same as before)
        let dfx_controller_principal = Principal::from_text("ju5k3-incuz-afpss-iopne-5tzfe-b466x-j4roy-owlyu-zq2pv-4dfjb-4ae").unwrap();

        // Create canister settings
        let create_canister_arg = CreateCanisterArgument {
            settings: Some(ic_cdk::api::management_canister::main::CanisterSettings {
                controllers: Some(vec![
                    ic_cdk::id(),
                    owner_principal,
                    dfx_controller_principal
                ]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                log_visibility: None,     
                wasm_memory_limit: None,  
            }),
        };

        // Ensure cycles value is converted to u128
        let cycles_to_use = cycles as u128;

        debug_log!("Cycles to use: {}", cycles_to_use);

        // Create the canister
        match create_canister(create_canister_arg, cycles_to_use).await {
            Ok((canister_id_record,)) => {
                let drive_canister_id = canister_id_record.canister_id;

                debug_log!("canister id deployed: {}", drive_canister_id);
                
                // Read WASM module
                const DRIVE_WASM: &[u8] = include_bytes!("../../../../../target/wasm32-unknown-unknown/release/canisters_official_backend.wasm");
                
                // Split WASM into chunks of 1MB each to stay safely under the 2MB limit
                const CHUNK_SIZE: usize = 1_000_000; // 1MB
                
                debug_log!("WASM module size: {} bytes", DRIVE_WASM.len());
                
                // Prepare chunks
                let mut chunks: Vec<Vec<u8>> = Vec::new();
                let mut chunk_hashes = Vec::new();
                
                let mut offset = 0;
                while offset < DRIVE_WASM.len() {
                    let end = std::cmp::min(offset + CHUNK_SIZE, DRIVE_WASM.len());
                    let chunk = DRIVE_WASM[offset..end].to_vec();
                    
                    // Upload chunk
                    debug_log!("Uploading chunk {} of size {} bytes", chunks.len(), chunk.len());
                    
                    let upload_arg = UploadChunkArgument {
                        canister_id: drive_canister_id,
                        chunk: chunk.clone(),
                    };
                    
                    match ic_cdk::api::management_canister::main::upload_chunk(upload_arg).await {
                        Ok((chunk_hash,)) => {
                            chunk_hashes.push(chunk_hash);
                            chunks.push(chunk);
                            debug_log!("Chunk uploaded successfully");
                        },
                        Err(e) => {
                            debug_log!("Failed to upload chunk: {:?}", e);
                            return Err(format!("Failed to upload WASM chunk: {:?}", e));
                        }
                    }
                    
                    offset = end;
                }
                
                debug_log!("All chunks uploaded, total chunks: {}", chunks.len());
                
                // Calculate WASM module hash (SHA-256)
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(DRIVE_WASM);
                let wasm_module_hash = hasher.finalize().to_vec();
                
                // Create SpawnInitArgs for the canister
                let init_args = SpawnInitArgs {
                    owner: owner_icp_principal,
                    title,
                    owner_name,
                    note,
                    spawn_redeem_code: Some(spawn_redeem_code),
                };

                // Encode initialization arguments
                let arg = match Encode!(&Option::<SpawnInitArgs>::Some(init_args)) {
                    Ok(a) => a,
                    Err(e) => return Err(format!("Failed to encode init arguments: {:?}", e)),
                };
                
                debug_log!("Encoded initialization arguments");

                // Install chunked code arguments
                let install_chunked_code_arg = InstallChunkedCodeArgument {
                    mode: CanisterInstallMode::Install,
                    target_canister: drive_canister_id,
                    store_canister: None, // Use the target canister as the storage canister
                    chunk_hashes_list: chunk_hashes,
                    wasm_module_hash,
                    arg,
                };

                debug_log!("Installing chunked code...");

                // Install the chunked code
                match install_chunked_code(install_chunked_code_arg).await {
                    Ok(()) => {
                        debug_log!("Code installed successfully");
                        Ok(drive_canister_id.to_string())
                    },
                    Err(e) => {
                        debug_log!("Failed to install chunked code: {:?}", e);
                        Err(format!("Failed to install chunked code: {:?}", e))
                    }
                }
            },
            Err(e) => {
                debug_log!("Failed to create canister: {:?}", e);
                Err(format!("Failed to create canister: {:?}", e))
            }
        }
    }

    // Format ISO8601 timestamp
    fn format_iso8601(time: u64) -> String {
        let nanoseconds = time as i64;
        let seconds = nanoseconds / 1_000_000_000;
        let nanos_remainder = nanoseconds % 1_000_000_000;
        
        let dt = time::OffsetDateTime::from_unix_timestamp(seconds)
            .unwrap()
            .saturating_add(time::Duration::nanoseconds(nanos_remainder));
        
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            dt.year(), dt.month() as u8, dt.day(),
            dt.hour(), dt.minute(), dt.second())
    }

    fn create_response(status_code: StatusCode, body: Vec<u8>) -> HttpResponse<'static> {
        HttpResponse::builder()
            .with_status_code(status_code)
            .with_headers(vec![
                ("content-type".to_string(), "application/json".to_string()),
                (
                    "strict-transport-security".to_string(),
                    "max-age=31536000; includeSubDomains".to_string(),
                ),
                ("x-content-type-options".to_string(), "nosniff".to_string()),
                ("referrer-policy".to_string(), "no-referrer".to_string()),
                (
                    "cache-control".to_string(),
                    "no-store, max-age=0".to_string(),
                ),
                ("pragma".to_string(), "no-cache".to_string()),
            ])
            .with_body(body)
            .build()
    }
}