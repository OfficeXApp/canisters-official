// src/rest/giftcards_refuel/handler.rs

pub mod giftcards_handlers {
    use std::{thread::sleep, time::Duration};
    use crate::core::api::helpers::is_local_environment;
    use crate::core::api::uuid::format_drive_id;
    use crate::core::state::giftcards_refuel::state::state::DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID;
    use crate::core::state::giftcards_refuel::types::FactoryRefuelHistoryRecord;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuelIDVec;
    use crate::core::state::giftcards_spawnorg::state::state::OWNER_ID;
    use crate::core::state::giftcards_spawnorg::types::DriveID;
    use crate::core::state::giftcards_spawnorg::types::DriveRESTUrlEndpoint;
    use crate::core::state::giftcards_spawnorg::types::FactorySpawnHistoryRecord;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuelID;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuel;
    use crate::core::types::ICPPrincipalString;
    use crate::core::types::PublicKeyICP;
    use crate::rest::giftcards_refuel::types::RedeemGiftcardRefuelResult;
    use crate::rest::giftcards_refuel::types::RedeemGiftcardRefuelData;
    
    use crate::{
        core::{
            api::uuid::{format_user_id, generate_uuidv4}, 
            state::giftcards_refuel::{
                    state::state::{HISTORICAL_GIFTCARDS_REFUELS, USER_TO_GIFTCARDS_REFUEL_HASHTABLE, GIFTCARD_REFUEL_BY_ID},
                    
                }, 
            types::{IDPrefix, UserID}
        }, 
        debug_log, 
        rest::{
            auth::{authenticate_request, create_auth_error_response}, giftcards_refuel::types::{
                CreateGiftcardRefuelRequestBody, CreateGiftcardRefuelResponse, DeleteGiftcardRefuelRequestBody, DeleteGiftcardRefuelResponse, DeletedGiftcardRefuelData, ErrorResponse, GetGiftcardRefuelResponse, ListGiftcardRefuelsRequestBody, ListGiftcardRefuelsResponse, ListGiftcardRefuelsResponseData, RedeemGiftcardRefuelResponse, SortDirection, UpdateGiftcardRefuelRequestBody, UpdateGiftcardRefuelResponse, UpsertGiftcardRefuelRequestBody
            }
        }, 
    };
    use candid::Principal;
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
        let requested_id = GiftcardRefuelID(params.get("giftcard_id").unwrap().to_string());

        // Get the requested giftcard
        let giftcard = GIFTCARD_REFUEL_BY_ID.with(|store| {
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
                    GetGiftcardRefuelResponse::ok(&v).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "GiftcardRefuel not found".to_string()).encode()
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
        let request_body: ListGiftcardRefuelsRequestBody = match serde_json::from_slice(body) {
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
        let total_count = HISTORICAL_GIFTCARDS_REFUELS.with(|historical_ids| {
            historical_ids.borrow().len()
        });

        // If there are no giftcards, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListGiftcardRefuelsResponse::ok(&ListGiftcardRefuelsResponseData {
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

        HISTORICAL_GIFTCARDS_REFUELS.with(|historical_ids| {
            let historical_ids = historical_ids.borrow();
            GIFTCARD_REFUEL_BY_ID.with(|store| {
                let store = store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_giftcards.len() < request_body.page_size && current_idx < total_count as usize {
                            if let Some(giftcard) = store.get(&historical_ids.get(current_idx as u64).unwrap()) {
                                // Apply filters if any
                                if request_body.filters.is_empty() || 
                                   (giftcard.note.to_lowercase().contains(&request_body.filters.to_lowercase())) {
                                    filtered_giftcards.push(giftcard.clone());
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
                            if let Some(giftcard) = store.get(&historical_ids.get(current_idx as u64).unwrap()) {
                                // Apply filters if any
                                if request_body.filters.is_empty() || 
                                   (giftcard.note.to_lowercase().contains(&request_body.filters.to_lowercase())) {
                                    filtered_giftcards.push(giftcard.clone());
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
            ListGiftcardRefuelsResponse::ok(&ListGiftcardRefuelsResponseData {
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

        if let Ok(req) = serde_json::from_slice::<UpsertGiftcardRefuelRequestBody>(body) {
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
                UpsertGiftcardRefuelRequestBody::Create(create_req) => {            
                    // Create new giftcard
                    let current_time = ic_cdk::api::time() / 1_000_000;
                    let new_giftcard = GiftcardRefuel {
                        id: GiftcardRefuelID(generate_uuidv4(IDPrefix::GiftcardRefuel)),
                        usd_revenue_cents: create_req.usd_revenue_cents,
                        note: create_req.note,
                        gas_cycles_included: create_req.gas_cycles_included,
                        timestamp_ms: current_time,
                        external_id: create_req.external_id,
                        redeemed: false,
                    };
            
                    // Add to GIFTCARD_REFUEL_BY_ID
                    GIFTCARD_REFUEL_BY_ID.with(|store| {
                        store.borrow_mut().insert(new_giftcard.id.clone(), new_giftcard.clone());
                    });
            
                    // Add to USER_TO_GIFTCARDS_REFUEL_HASHTABLE for the owner
                    let owner_id = OWNER_ID.with(|id| id.borrow().get().clone());
                    USER_TO_GIFTCARDS_REFUEL_HASHTABLE.with(|store| {
                        let mut store = store.borrow_mut();
                        let mut vec = match store.get(&owner_id) {
                            Some(v) => v.clone(),
                            None => GiftcardRefuelIDVec::new(),
                        };
                        vec.push(new_giftcard.id.clone());
                        store.insert(owner_id, vec);
                    });
                    
                    // Add to HISTORICAL_GIFTCARDS_REFUELS
                    HISTORICAL_GIFTCARDS_REFUELS.with(|giftcards| {
                        giftcards.borrow_mut().push(&new_giftcard.id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        CreateGiftcardRefuelResponse::ok(&new_giftcard).encode()
                    )  
                },
                UpsertGiftcardRefuelRequestBody::Update(update_req) => {
                    // Get the giftcard to update
                    let giftcard_id = GiftcardRefuelID(update_req.id);
                    let mut giftcard = match GIFTCARD_REFUEL_BY_ID.with(|store| store.borrow().get(&giftcard_id).clone()) {
                        Some(v) => v,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::err(404, "GiftcardRefuel not found".to_string()).encode()
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
            
                    // Update the giftcard in GIFTCARD_REFUEL_BY_ID
                    GIFTCARD_REFUEL_BY_ID.with(|store| {
                        store.borrow_mut().insert(giftcard.id.clone(), giftcard.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateGiftcardRefuelResponse::ok(&giftcard).encode()
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
        let delete_request = match serde_json::from_slice::<DeleteGiftcardRefuelRequestBody>(body) {
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
        let giftcard_id = GiftcardRefuelID(delete_request.id.clone());
        let _giftcard = match GIFTCARD_REFUEL_BY_ID.with(|store| store.borrow().get(&giftcard_id).clone()) {
            Some(v) => v,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "GiftcardRefuel not found".to_string()).encode()
                )
            }
        };

        // Remove from GIFTCARD_REFUEL_BY_ID
        GIFTCARD_REFUEL_BY_ID.with(|store| {
            store.borrow_mut().remove(&giftcard_id);
        });

        // Remove from USER_TO_GIFTCARDS_REFUEL_HASHTABLE
        let owner_id = OWNER_ID.with(|id| (*id.borrow().get()).clone());
        USER_TO_GIFTCARDS_REFUEL_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(existing_ids) = store.get(&owner_id) {
                let mut updated_ids = existing_ids.clone();
                updated_ids.items.retain(|id| id != &giftcard_id);
                
                // Only update if there are remaining items
                if updated_ids.items.is_empty() {
                    store.remove(&owner_id);
                } else {
                    store.insert(owner_id.clone(), updated_ids);
                }
            }
        });
        
        // Note: We intentionally do NOT remove from HISTORICAL_GIFTCARDS_REFUELS
        // This preserves the historical record even if a giftcard is deleted

        // Return success response
        create_response(
            StatusCode::OK,
            DeleteGiftcardRefuelResponse::ok(&DeletedGiftcardRefuelData {
                id: delete_request.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn redeem_giftcard_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, _params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming redeem giftcard request: {}", request.url());
    
        // Parse request body
        let body: &[u8] = request.body();
        
        let redeem_request = match serde_json::from_slice::<RedeemGiftcardRefuelData>(body) {
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
        let giftcard = match GIFTCARD_REFUEL_BY_ID.with(|store| store.borrow().get(&giftcard_id).clone()) {
            Some(v) => v,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "GiftcardRefuel not found".to_string()).encode()
                )
            }
        };
    
        // Check if giftcard is already redeemed
        if giftcard.redeemed {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "GiftcardRefuel already redeemed".to_string()).encode()
            );
        }
    
        // Convert ICP principal to Principal
        let recipient_principal = match Principal::from_text(&redeem_request.icp_principal) {
            Ok(p) => p,
            Err(_) => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid ICP principal".to_string()).encode()
                )
            }
        };
    
        // Generate a unique redeem code using timestamp
        let redeem_code = format!("REDEEM_{}", ic_cdk::api::time());
        let current_time = ic_cdk::api::time() / 1_000_000;
    
        // Deposit cycles to the recipient's principal
        match deposit_cycles(recipient_principal, giftcard.gas_cycles_included).await {
            Ok(_) => {
                // Update giftcard as redeemed
                GIFTCARD_REFUEL_BY_ID.with(|store| {
                    let mut giftcard = giftcard.clone();
                    giftcard.redeemed = true;
                    store.borrow_mut().insert(giftcard_id.clone(), giftcard);
                });
    
                // Store the redemption history
                let user_id = format_user_id(&redeem_request.icp_principal);
                
                // Create a record of the transaction
                let history_record = FactoryRefuelHistoryRecord {
                    note: format!("Redeemed giftcard {} by {}, deposited {} cycles into canister {}", giftcard_id, user_id, giftcard.gas_cycles_included, recipient_principal),
                    giftcard_id: giftcard_id.clone(),
                    gas_cycles_included: giftcard.gas_cycles_included,
                    timestamp_ms: current_time,
                    icp_principal: ICPPrincipalString(PublicKeyICP(redeem_request.icp_principal.clone())),
                };
    
                // Store the redemption history
                DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID.with(|records| {
                    records.borrow_mut().insert(giftcard_id.clone(), history_record.clone());
                });
    
                // Add to USER_TO_GIFTCARDS_REFUEL_HASHTABLE for the user
                USER_TO_GIFTCARDS_REFUEL_HASHTABLE.with(|store| {
                    let mut store = store.borrow_mut();
                    let mut vec = match store.get(&user_id) {
                        Some(v) => v.clone(),
                        None => GiftcardRefuelIDVec::new(),
                    };
                    vec.push(giftcard_id.clone());
                    store.insert(user_id, vec);
                });
    
                let redeem_giftcard_result = RedeemGiftcardRefuelResult {
                    giftcard_id: giftcard_id,
                    icp_principal: redeem_request.icp_principal,
                    redeem_code: redeem_code,
                    timestamp_ms: current_time,
                };
    
                // Return success response
                create_response(
                    StatusCode::OK,
                    RedeemGiftcardRefuelResponse::ok(&redeem_giftcard_result).encode()
                )
            },
            Err(e) => {
                create_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::err(500, format!("Failed to deposit cycles: {}", e)).encode()
                )
            }
        }
    }
    
    // Helper function to deposit cycles to a principal
    async fn deposit_cycles(recipient: Principal, amount: u64) -> Result<(), String> {
        use ic_cdk::api::management_canister::main::{deposit_cycles, CanisterIdRecord};
        
        // Convert cycles to u128
        let cycles_amount = amount as u128;
        
        match deposit_cycles(CanisterIdRecord { canister_id: recipient }, cycles_amount).await {
            Ok(()) => {
                debug_log!("Successfully deposited {} cycles to principal {}", amount, recipient);
                Ok(())
            },
            Err(e) => {
                debug_log!("Failed to deposit cycles: {:?}", e);
                Err(format!("Failed to deposit cycles: {:?}", e))
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