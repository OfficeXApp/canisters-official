// src/rest/vouchers/handler.rs

pub mod vouchers_handlers {
    use std::{thread::sleep, time::Duration};
    use crate::core::state::vouchers::types::DriveID;
    use crate::core::state::vouchers::types::DriveRESTUrlEndpoint;
    use crate::core::state::vouchers::types::FactorySpawnHistoryRecord;
    use crate::core::state::vouchers::types::VoucherID;
    use crate::core::state::vouchers::types::Voucher;
    use crate::rest::vouchers::types::RedeemVoucherData;
    use crate::rest::vouchers::types::SpawnInitArgs;
    use crate::{
        core::{
            api::uuid::{format_user_id, generate_unique_id}, 
            state::vouchers::{
                    state::state::{HISTORICAL_VOUCHERS, OWNER_ID, USER_TO_VOUCHERS_HASHTABLE, VOUCHER_BY_ID},
                    
                }, 
            types::{IDPrefix, UserID}
        }, 
        debug_log, 
        rest::{
            auth::{authenticate_request, create_auth_error_response}, vouchers::types::{
                CreateVoucherRequestBody, CreateVoucherResponse, DeleteVoucherRequestBody, DeleteVoucherResponse, DeletedVoucherData, ErrorResponse, GetVoucherResponse, ListVouchersRequestBody, ListVouchersResponse, ListVouchersResponseData, RedeemVoucherResponse, SortDirection, UpdateVoucherRequestBody, UpdateVoucherResponse, UpsertVoucherRequestBody
            }
        }, 
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        note: Option<String>,
    }

    pub async fn get_voucher_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Get the requested voucher ID from params
        let requested_id = VoucherID(params.get("voucher_id").unwrap().to_string());

        // Get the requested voucher
        let voucher = VOUCHER_BY_ID.with(|store| {
            store.borrow().get(&requested_id).cloned()
        });

        // Check authorization (only owner can view vouchers)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        if !is_owner {
            return create_auth_error_response();
        }
 
        match voucher {
            Some(v) => {
                create_response(
                    StatusCode::OK,
                    GetVoucherResponse::ok(&v).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Voucher not found".to_string()).encode()
            ),
        }
    }

    pub async fn list_vouchers_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming request: {}", request.url());

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check authorization - only owner can list all vouchers
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body = request.body();
        let request_body: ListVouchersRequestBody = match serde_json::from_slice(body) {
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
        let cursor_up = if let Some(cursor) = request_body.cursor_up {
            match cursor.parse::<usize>() {
                Ok(idx) => Some(idx),
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid cursor_up format".to_string()).encode()
                ),
            }
        } else {
            None
        };

        let cursor_down = if let Some(cursor) = request_body.cursor_down {
            match cursor.parse::<usize>() {
                Ok(idx) => Some(idx),
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid cursor_down format".to_string()).encode()
                ),
            }
        } else {
            None
        };

        // Get total count from historical vouchers
        let total_count = HISTORICAL_VOUCHERS.with(|historical_ids| {
            historical_ids.borrow().len()
        });

        // If there are no vouchers, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListVouchersResponse::ok(&ListVouchersResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    cursor_up: None,
                    cursor_down: None,
                }).encode()
            );
        }

        // Determine starting point based on cursors
        let start_index = if let Some(up) = cursor_up {
            up.min(total_count - 1)
        } else if let Some(down) = cursor_down {
            down.min(total_count - 1)
        } else {
            match request_body.direction {
                SortDirection::Asc => 0,
                SortDirection::Desc => total_count - 1,
            }
        };

        // Get vouchers with pagination and filtering
        let mut filtered_vouchers = Vec::new();
        let mut processed_count = 0;

        HISTORICAL_VOUCHERS.with(|historical_ids| {
            let historical_ids = historical_ids.borrow();
            VOUCHER_BY_ID.with(|store| {
                let store = store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_vouchers.len() < request_body.page_size && current_idx < total_count {
                            if let Some(voucher) = store.get(&historical_ids[current_idx]) {
                                // Apply filters if any
                                if request_body.filters.is_empty() || 
                                   (voucher.note.to_lowercase().contains(&request_body.filters.to_lowercase())) {
                                    filtered_vouchers.push(voucher.clone());
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
                        while filtered_vouchers.len() < request_body.page_size && current_idx < total_count {
                            if let Some(voucher) = store.get(&historical_ids[current_idx]) {
                                // Apply filters if any
                                if request_body.filters.is_empty() || 
                                   (voucher.note.to_lowercase().contains(&request_body.filters.to_lowercase())) {
                                    filtered_vouchers.push(voucher.clone());
                                }
                            }
                            current_idx += 1;
                            processed_count = current_idx - start_index;
                            if current_idx >= total_count {
                                break;
                            }
                        }
                    }
                }
            });
        });

        // Calculate next cursors based on direction and current position
        let (cursor_up, cursor_down) = match request_body.direction {
            SortDirection::Desc => {
                let next_up = if start_index < total_count - 1 {
                    Some((start_index + 1).to_string())
                } else {
                    None
                };
                let next_down = if processed_count > 0 && start_index >= processed_count {
                    Some((start_index - processed_count).to_string())
                } else {
                    None
                };
                (next_up, next_down)
            },
            SortDirection::Asc => {
                let next_up = if processed_count > 0 && start_index + processed_count < total_count {
                    Some((start_index + processed_count).to_string())
                } else {
                    None
                };
                let next_down = if start_index > 0 {
                    Some((start_index - 1).to_string())
                } else {
                    None
                };
                (next_up, next_down)
            }
        };

        create_response(
            StatusCode::OK,
            ListVouchersResponse::ok(&ListVouchersResponseData {
                items: filtered_vouchers.clone(),
                page_size: filtered_vouchers.len(),
                total: total_count,
                cursor_up,
                cursor_down,
            }).encode()
        )
    }

    pub async fn upsert_voucher_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check if requester is owner (only owner can create/update vouchers)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            return create_auth_error_response();
        }
    
        // Parse request body
        let body: &[u8] = request.body();

        if let Ok(req) = serde_json::from_slice::<UpsertVoucherRequestBody>(body) {
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
                UpsertVoucherRequestBody::Create(create_req) => {            
                    // Create new voucher
                    let current_time = ic_cdk::api::time();
                    let new_voucher = Voucher {
                        id: VoucherID(generate_unique_id(IDPrefix::Voucher, "")),
                        usd_revenue_cents: create_req.usd_revenue_cents,
                        note: create_req.note,
                        gas_cycles_included: create_req.gas_cycles_included,
                        timestamp_ms: current_time,
                        external_id: create_req.external_id,
                        redeemed: false,
                    };
            
                    // Add to VOUCHER_BY_ID
                    VOUCHER_BY_ID.with(|store| {
                        store.borrow_mut().insert(new_voucher.id.clone(), new_voucher.clone());
                    });
            
                    // Add to USER_TO_VOUCHERS_HASHTABLE for the owner
                    let owner_id = OWNER_ID.with(|id| id.borrow().clone());
                    USER_TO_VOUCHERS_HASHTABLE.with(|store| {
                        store.borrow_mut()
                            .entry(owner_id)
                            .or_insert_with(Vec::new)
                            .push(new_voucher.id.clone());
                    });
                    
                    // Add to HISTORICAL_VOUCHERS
                    crate::core::state::vouchers::state::state::HISTORICAL_VOUCHERS.with(|vouchers| {
                        vouchers.borrow_mut().push(new_voucher.id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        CreateVoucherResponse::ok(&new_voucher).encode()
                    )  
                },
                UpsertVoucherRequestBody::Update(update_req) => {
                    // Get the voucher to update
                    let voucher_id = VoucherID(update_req.id);
                    let mut voucher = match VOUCHER_BY_ID.with(|store| store.borrow().get(&voucher_id).cloned()) {
                        Some(v) => v,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::err(404, "Voucher not found".to_string()).encode()
                        ),
                    };

                    // Update only the fields that were provided
                    if let Some(notes) = update_req.notes {
                        voucher.note = notes;
                    }
                    if let Some(usd_revenue_cents) = update_req.usd_revenue_cents {
                        voucher.usd_revenue_cents = usd_revenue_cents;
                    }
                    if let Some(gas_cycles_included) = update_req.gas_cycles_included {
                        voucher.gas_cycles_included = gas_cycles_included;
                    }
                    if let Some(external_id) = update_req.external_id {
                        voucher.external_id = external_id;
                    }
            
                    // Update the voucher in VOUCHER_BY_ID
                    VOUCHER_BY_ID.with(|store| {
                        store.borrow_mut().insert(voucher.id.clone(), voucher.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateVoucherResponse::ok(&voucher).encode()
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

    pub async fn delete_voucher_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming request: {}", request.url());

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check if requester is owner (only owner can delete vouchers)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body: &[u8] = request.body();
        
        debug_log!("Incoming request body: {}", String::from_utf8_lossy(request.body()));
        let delete_request = match serde_json::from_slice::<DeleteVoucherRequestBody>(body) {
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

        // Get the voucher to be deleted
        let voucher_id = VoucherID(delete_request.id.clone());
        let _voucher = match VOUCHER_BY_ID.with(|store| store.borrow().get(&voucher_id).cloned()) {
            Some(v) => v,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "Voucher not found".to_string()).encode()
                )
            }
        };

        // Remove from VOUCHER_BY_ID
        VOUCHER_BY_ID.with(|store| {
            store.borrow_mut().remove(&voucher_id);
        });

        // Remove from USER_TO_VOUCHERS_HASHTABLE
        let owner_id = OWNER_ID.with(|id| id.borrow().clone());
        USER_TO_VOUCHERS_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(voucher_ids) = store.get_mut(&owner_id) {
                voucher_ids.retain(|id| id != &voucher_id);
                // If this was the last voucher for the user, remove the user entry
                if voucher_ids.is_empty() {
                    store.remove(&owner_id);
                }
            }
        });
        
        // Note: We intentionally do NOT remove from HISTORICAL_VOUCHERS
        // This preserves the historical record even if a voucher is deleted

        // Return success response
        create_response(
            StatusCode::OK,
            DeleteVoucherResponse::ok(&DeletedVoucherData {
                id: delete_request.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn redeem_voucher_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming redeem voucher request: {}", request.url());
    
        // // Authenticate request
        // let requester_api_key = match authenticate_request(request) {
        //     Some(key) => key,
        //     None => return create_auth_error_response(),
        // };
    
        // Parse request body
        let body: &[u8] = request.body();
        
        let redeem_request = match serde_json::from_slice::<RedeemVoucherData>(body) {
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
    
        // Get the voucher to be redeemed
        let voucher_id = redeem_request.id.clone();
        let voucher = match VOUCHER_BY_ID.with(|store| store.borrow().get(&voucher_id).cloned()) {
            Some(v) => v,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "Voucher not found".to_string()).encode()
                )
            }
        };
    
        // Check if voucher is already redeemed
        if voucher.redeemed {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Voucher already redeemed".to_string()).encode()
            );
        }
    
        // Generate a unique redeem code using timestamp
        let redeem_code = format!("REDEEM_{}", ic_cdk::api::time());
    
        // Convert ICP principal to UserID format
        let owner_id = format_user_id(&redeem_request.owner_icp_principal);
    
        // Create note for the factory spawn
        let current_time = ic_cdk::api::time();
        let time_iso = format_iso8601(current_time);
        
        // Deploy the canister using IC management canister
        let deployed_canister = match deploy_drive_canister(
            redeem_request.owner_icp_principal.clone(),
            redeem_request.nickname.clone(),
            redeem_code.clone(),
            Some(format!("voucher {} was redeemed to spawn drive with {} cycles, owned by {}, on timestamp_ms {} {}", 
                voucher_id.0, voucher.gas_cycles_included, owner_id.0, current_time, time_iso)),
            voucher.gas_cycles_included,
        ).await {
            Ok(canister_id) => canister_id,
            Err(e) => {
                return create_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::err(500, format!("Failed to deploy canister: {}", e)).encode()
                )
            }
        };
    
        // Wait a brief moment to ensure the canister is ready for HTTP calls
        // This may need to be adjusted based on deployment timing
        sleep(Duration::from_secs(2));
    
        // Now make a call to redeem_spawn to get the API credentials
        let admin_login_password = match get_spawn_login_password(&deployed_canister, &redeem_code).await {
            Ok(creds) => creds,
            Err(e) => {
                debug_log!("Warning: Failed to fetch API credentials: {}", e);
                format!("API credentials could not be automatically retrieved. Please use the redeem_spawn endpoint with code: {}", redeem_code)
            }
        };
    
        // Create a record of the deployment
        let version = crate::core::state::vouchers::state::state::VERSION.with(|v| v.borrow().clone());
        
        // Get appropriate URL endpoint for the deployed canister
        let endpoint = DriveRESTUrlEndpoint(format!("https://{}.icp0.io", deployed_canister));
        
        let history_record = FactorySpawnHistoryRecord {
            owner_id: owner_id.clone(),
            drive_id: DriveID(deployed_canister.clone()),
            endpoint: endpoint.clone(),
            version,
            note: voucher.note.clone(),
            voucher_id: voucher_id.clone(),
            gas_cycles_included: voucher.gas_cycles_included,
            timestamp_ms: current_time,
            admin_login_password: admin_login_password.clone(),
        };
    
        // Update voucher as redeemed
        VOUCHER_BY_ID.with(|store| {
            let mut voucher = voucher.clone();
            voucher.redeemed = true;
            store.borrow_mut().insert(voucher_id.clone(), voucher);
        });
    
        // Store the deployment history
        crate::core::state::vouchers::state::state::DEPLOYMENTS_BY_VOUCHER_ID.with(|records| {
            records.borrow_mut().insert(voucher_id.clone(), history_record.clone());
        });
    
        // Add to DRIVE_TO_VOUCHER_HASHTABLE
        crate::core::state::vouchers::state::state::DRIVE_TO_VOUCHER_HASHTABLE.with(|map| {
            map.borrow_mut().insert(DriveID(deployed_canister.clone()), voucher_id.clone());
        });
    
        // Add to USER_TO_VOUCHERS_HASHTABLE for the owner
        USER_TO_VOUCHERS_HASHTABLE.with(|store| {
            store.borrow_mut()
                .entry(owner_id.clone())
                .or_insert_with(Vec::new)
                .push(voucher_id.clone());
        });
    
        // Return success response
        create_response(
            StatusCode::OK,
            RedeemVoucherResponse::ok(&history_record).encode()
        )
    }
    
    // Helper function to get API credentials via redeem_spawn endpoint
    async fn get_spawn_login_password(canister_id: &str, redeem_code: &str) -> Result<String, String> {
        use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
        use serde_json::{json, Value};
    
        let url = format!("https://{}.icp0.io/v1/default/organization/redeem_spawn", canister_id);
        let is_local = crate::core::api::helpers::is_local_environment();
        
        if is_local {
            // Get the port for local development
            let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
            // In local development, URLs are typically structured like:
            // http://{canister_id}.localhost:{port}
            let url = format!("http://{}.localhost:{}/v1/default/organization/redeem_spawn", canister_id, port);
        }
    
        // Prepare the request body
        let request_body = json!({
            "redeem_code": redeem_code
        }).to_string();
    
        // Prepare headers
        let headers = vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ];
    
        // Create the HTTP request
        let request = CanisterHttpRequestArgument {
            url,
            method: HttpMethod::POST,
            headers,
            body: Some(request_body.into_bytes()),
            max_response_bytes: Some(5 * 1024), // 5KB should be enough
            transform: None,
        };
    
        // Make the HTTP request
        let cycles: u128 = 100_000_000_000; // 100 billion cycles
        
        match http_request(request, cycles).await {
            Ok((response,)) => {
                // Check if the response was successful
                let status_u16 = match response.status.0.try_into() {
                    Ok(n) => n,
                    Err(_) => 500, // Default to 500 if conversion fails
                };
                if status_u16 >= 200 && status_u16 < 300 {
                    // Parse the response JSON
                    match serde_json::from_slice::<Value>(&response.body) {
                        Ok(json_response) => {
                            // Try to extract the admin_login_password
                            if let Some(data) = json_response.get("ok").and_then(|ok| ok.get("data")) {
                                if let Some(password) = data.get("admin_login_password").and_then(|pw| pw.as_str()) {
                                    return Ok(password.to_string());
                                }
                            }
                            Err("Could not find admin_login_password in response".to_string())
                        },
                        Err(e) => Err(format!("Failed to parse response JSON: {}", e)),
                    }
                } else {
                    Err(format!("HTTP request failed with status {}: {}", 
                    status_u16,
                        String::from_utf8_lossy(&response.body)))
                }
            },
            Err((code, msg)) => Err(format!("HTTP request failed: {:?} - {}", code, msg))
        }
    }
    
    // Helper function to deploy a drive canister
    async fn deploy_drive_canister(
        owner_icp_principal: String, 
        nickname: Option<String>, 
        spawn_redeem_code: String,
        note: Option<String>,
        cycles: u64
    ) -> Result<String, String> {
        use ic_cdk::api::management_canister::main::{
            create_canister, install_code, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument,
        };
        use candid::{Encode, Principal};
    
        // Convert owner ID to Principal
        let owner_principal = match Principal::from_text(&owner_icp_principal) {
            Ok(p) => p,
            Err(_) => return Err("Invalid owner ICP principal".to_string()),
        };
    
        debug_log!("Creating drive for owner: {}", owner_icp_principal);
    
        // Create canister settings
        let create_canister_arg = CreateCanisterArgument {
            settings: Some(ic_cdk::api::management_canister::main::CanisterSettings {
                controllers: Some(vec![ic_cdk::id(), owner_principal]),
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
    
        // Create the canister
        match create_canister(create_canister_arg, cycles_to_use).await {
            Ok((canister_id_record,)) => {
                let drive_canister_id = canister_id_record.canister_id;
                
                // Read WASM module from path
                const DRIVE_WASM: &[u8] = include_bytes!("../../../../../target/wasm32-unknown-unknown/release/canisters_official_backend.wasm");
    
                // Create SpawnInitArgs for the canister
                let init_args = SpawnInitArgs {
                    owner: owner_icp_principal,
                    nickname,
                    note,
                    spawn_redeem_code: Some(spawn_redeem_code),
                };
    
                // Encode initialization arguments
                let arg = match Encode!(&Option::<SpawnInitArgs>::Some(init_args)) {
                    Ok(a) => a,
                    Err(e) => return Err(format!("Failed to encode init arguments: {:?}", e)),
                };
                
                debug_log!("Encoded initialization arguments");
    
                // Install code arguments
                let install_code_arg = InstallCodeArgument {
                    mode: CanisterInstallMode::Install,
                    canister_id: drive_canister_id,
                    wasm_module: DRIVE_WASM.to_vec(),
                    arg,
                };
    
                debug_log!("Installing code...");
    
                // Install the code
                match install_code(install_code_arg).await {
                    Ok(()) => {
                        debug_log!("Code installed successfully");
                        Ok(drive_canister_id.to_string())
                    },
                    Err(e) => {
                        debug_log!("Failed to install code: {:?}", e);
                        Err(format!("Failed to install code: {:?}", e))
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