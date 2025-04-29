// src/rest/organization/handler.rs


pub mod drives_handlers {
    use crate::{
        core::{api::{permissions::{directory::{can_user_access_directory_permission, check_directory_permissions}, system::{can_user_access_system_permission, check_system_permissions}}, replay::diff::{apply_state_diff, convert_state_to_serializable, safely_apply_diffs, snapshot_entire_state, snapshot_poststate, snapshot_prestate}, uuid::generate_uuidv4, webhooks::organization::{fire_org_inbox_new_notif_webhook, fire_superswap_user_webhook, get_org_inbox_webhooks, get_superswap_user_webhooks}}, state::{api_keys::state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, disks::state::state::{DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, drives::{state::state::{superswap_userid, update_external_id_mapping, CANISTER_ID, DRIVES_BY_ID_HASHTABLE, DRIVES_BY_TIME_LIST, DRIVE_ID, DRIVE_STATE_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, EXTERNAL_ID_MAPPINGS, OWNER_ID, SPAWN_NOTE, SPAWN_REDEEM_CODE, TRANSFER_OWNER_ID, URL_ENDPOINT}, types::{Drive, DriveID, DriveRESTUrlEndpoint, DriveStateDiffID, ExternalID, ExternalPayload, InboxNotifID, SpawnRedeemCode}}, group_invites::state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, groups::state::state::{is_group_admin, GROUPS_BY_ID_HASHTABLE, GROUPS_BY_TIME_LIST}, labels::{state::{add_label_to_resource, parse_label_resource_id, remove_label_from_resource, validate_label_value}, types::{LabelOperationResponse, LabelResourceID}}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE}, types::{DirectoryPermissionType, PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, search::types::{SearchCategoryEnum, SearchResult}, webhooks::types::WebhookEventLabel}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::DirectoryResourceID, organization::types::{AboutDriveResponse, AboutDriveResponseData, ErrorResponse, ExternalIDsDriveRequestBody, ExternalIDsDriveResponse, ExternalIDsDriveResponseData, ExternalIDvsInternalIDMaps, GetWhoAmIResponse, InboxOrgRequestBody, InboxOrgResponse, InboxOrgResponseData, RedeemOrgRequestBody, RedeemOrgResponse, RedeemOrgResponseData, ReindexDriveRequestBody, ReindexDriveResponse, ReindexDriveResponseData, ReplayDriveRequestBody, ReplayDriveResponse, ReplayDriveResponseData, SearchDriveRequestBody, SearchDriveResponse, SearchDriveResponseData, SearchSortByEnum, SuperswapUserIDRequestBody, SuperswapUserIDResponse, SuperswapUserIDResponseData, TransferOwnershipDriveRequestBody, TransferOwnershipDriveResponse, TransferOwnershipResponseData, TransferOwnershipStatusEnum, WhoAmIReport}, webhooks::types::SortDirection}
        
    };
    use candid::Principal;
    use ic_cdk::api::management_canister::main::CanisterIdRecord;
    use ic_types::crypto::canister_threshold_sig::PublicKey;
    use serde_json::json;
    use crate::core::state::search::state::state::{raw_query,filter_search_results_by_permission};
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub async fn about_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        

        // Get the drive ID
        let drive_id = DRIVE_ID.with(|id| id.clone());
        
        // Check permissions for viewing the drive
        let has_drive_permission = if is_owner {
            true // Owner has all permissions
        } else {
            // Check specific drive record permission
            let drive_record_permissions = check_system_permissions(
                SystemResourceID::Record(SystemRecordIDEnum::Drive(drive_id.0.clone())),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            // Check table-level permissions
            let drive_table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Drives),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            // User needs View permission on either the specific drive or the drives table
            drive_record_permissions.contains(&SystemPermissionType::View) || 
            drive_table_permissions.contains(&SystemPermissionType::View)
        };
        
        if !has_drive_permission {
            return create_auth_error_response();
        }

        
        // Get drive information
        let organization_name = DRIVES_BY_ID_HASHTABLE.with(|drives| {
            DRIVE_ID.with(|drive_id| {
                drives.borrow()
                    .get(drive_id)
                    .map(|drive| drive.name.clone())
                    .unwrap_or_else(|| "Unnamed Organization".to_string())
            })
        });
        
        let drive_id = DRIVE_ID.with(|id| id.clone());
        let owner = OWNER_ID.with(|owner| owner.borrow().get().clone());
        let endpoint = URL_ENDPOINT.with(|url| url.borrow().get().0.clone());
        let canister_id = CANISTER_ID.with(|id| id.0.clone());
        
        // Get current cycle balance
        let gas_cycles = ic_cdk::api::canister_balance().to_string();

        // Get daily idle cycle burn rate and controllers using management canister
        let canister_id_principal = Principal::from_text(&canister_id).unwrap_or_else(|_| Principal::anonymous());
        
        let args = CanisterIdRecord {
            canister_id: canister_id_principal,
        };
        
        // Default values in case of error
        let mut daily_idle_cycle_burn_rate = "0".to_string();
        let mut controllers: Vec<String> = vec![];
        
        // Try to get canister status
        if let Ok(status_result) = ic_cdk::api::management_canister::main::canister_status(args).await {
            let status = status_result.0;
            
            // Update idle cycle burn rate if available
            daily_idle_cycle_burn_rate = status.idle_cycles_burned_per_day.to_string();
            
            // Update controllers if available
            controllers = status.settings.controllers
                .into_iter()
                .map(|principal| principal.to_string())
                .collect();
        }

        // Create response data
        let response_data = AboutDriveResponseData {
            gas_cycles,
            organization_name,
            organization_id: drive_id,
            owner,
            endpoint,
            canister_id,
            daily_idle_cycle_burn_rate,
            controllers,
        };
        create_response(
            StatusCode::OK,
            AboutDriveResponse::ok(&response_data).encode()
        )
    }
    
    pub async fn snapshot_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // temporarily disabled for testing
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        if !is_owner {
            return create_auth_error_response();
        }

        // debug_log!("Requester API Key, {:?}", requester_api_key);

        let snapshot = snapshot_entire_state();

        debug_log!(">> snapshot {:?}", snapshot);

        let serializable_state = convert_state_to_serializable(&snapshot);

        // Return the JSON response
        match serde_json::to_vec(&serializable_state) {
            Ok(json) => create_response(StatusCode::OK, json),
            Err(e) => {
                // Log detailed error information
                ic_cdk::println!("Failed to serialize state snapshot: {:?}", e);
                
                // Return a more informative error response to the client
                create_response(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    ErrorResponse::err(500, format!("Failed to serialize state: {}", e)).encode()
                )
            }
        }
        // match serde_json::to_vec(&snapshot) {
        //     Ok(json) => create_response(StatusCode::OK, json),
        //     Err(e) => {
        //         // Log detailed error information
        //         ic_cdk::println!("Failed to serialize state snapshot: {:?}", e);
                
        //         // Return a more informative error response to the client
        //         create_response(
        //             StatusCode::INTERNAL_SERVER_ERROR, 
        //             ErrorResponse::err(500, format!("Failed to serialize state: {}", e)).encode()
        //         )
        //     }
        // }
    }

    pub async fn replay_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        if !is_owner {
            return create_auth_error_response();
        }
    
        // Parse request body
        let body: &[u8] = request.body();
        let replay_request = match serde_json::from_slice::<ReplayDriveRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
        if let Err(validation_error) = replay_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }
    
        // Check if diffs are provided
        if replay_request.diffs.is_empty() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "No diffs provided for replay".to_string()).encode()
            );
        }
        
        // Take a snapshot for audit/logging
        let prestate = snapshot_prestate();
        
        // Apply diffs with validation using our safety function
        match safely_apply_diffs(&replay_request.diffs) {
            Ok((applied_count, last_diff_id)) => {
                // Only log if we actually applied diffs
                if applied_count > 0 {
                    // Get the timestamp from the last applied diff
                    let last_timestamp = replay_request.diffs.iter()
                        .find(|d| Some(d.id.clone()) == last_diff_id)
                        .map(|d| d.timestamp_ns)
                        .unwrap_or_default();
                    
                    // Determine direction for logging
                    let current_timestamp = DRIVE_STATE_TIMESTAMP_NS.with(|ts| *ts.borrow().get());
                    let direction_str = if replay_request.diffs[0].timestamp_ns < current_timestamp {
                        "backward"
                    } else {
                        "forward"
                    };
                    
                    // Log notes if provided
                    let notes_str = format!(
                        "{}: Replay {} diffs {} to timestamp {} - {}", 
                        requester_api_key.user_id,
                        applied_count,
                        direction_str,
                        last_timestamp,
                        replay_request.notes.clone().unwrap_or_default()
                    );
                    
                    snapshot_poststate(prestate, Some(notes_str));
                }
                
                // Prepare response data
                let response_data = ReplayDriveResponseData {
                    timestamp_ns: DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.borrow().get().clone()),
                    diffs_applied: applied_count,
                    checkpoint_diff_id: last_diff_id,
                    final_checksum: DRIVE_STATE_CHECKSUM.with(|cs| cs.borrow().get().clone()),
                };
                
                create_response(
                    StatusCode::OK,
                    ReplayDriveResponse::ok(&response_data).encode()
                )
            },
            Err(error_msg) => {
                // Return error (rollback already happened in safely_apply_diffs)
                create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, error_msg).encode()
                )
            }
        }
    }

    
    pub async fn search_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Check if user is owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        // Parse request body
        let body = request.body();
        let request_body: SearchDriveRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }
    
        // Check if search query is provided (already validated in validate_body, but double-checking)
        if request_body.query.trim().is_empty() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Search query cannot be empty".to_string()).encode()
            );
        }
    
        // Use the categories from the request or default to All if None or empty
        let categories = match &request_body.categories {
            Some(cats) if !cats.is_empty() => Some(cats.clone()),
            _ => Some(vec![SearchCategoryEnum::All])
        };
    
        // Get page size with default if not provided
        let page_size = request_body.page_size.unwrap_or_else(|| 50);
    
        // Get sort direction with default if not provided
        let direction = request_body.direction.unwrap_or_default();
    
        // Perform the search using the search module
        let max_edit_distance = 2; // Allow up to 2 character edits for fuzzy matching
        let search_results = raw_query(&request_body.query, max_edit_distance, categories);
        
        // Create a PermissionGranteeID from the requester's user ID for permission checks
        let grantee_id = PermissionGranteeID::User(requester_api_key.user_id.clone());
        
        // Filter results based on permissions
        let mut filtered_results = filter_search_results_by_permission(&search_results, &grantee_id, is_owner).await;
        
        // Sort results based on sort_by if provided
        if let Some(sort_by) = &request_body.sort_by {
            match sort_by {
                SearchSortByEnum::CreatedAt => {
                    // Sort by created_at timestamp
                    filtered_results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                },
                _ => {
                    // Sort by updated_at timestamp
                    filtered_results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                }
            }
        }
    
        // Reverse the sort if direction is Desc
        if direction == SortDirection::Desc {
            filtered_results.reverse();
        }
    
        // Get total count of filtered results
        let total_count = filtered_results.len();
    
        // If there are no results, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                SearchDriveResponse::ok(&SearchDriveResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    direction,
                    cursor: None,
                }).encode()
            );
        }
    
        // Parse cursor if provided
        let start_cursor = if let Some(cursor) = &request_body.cursor {
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
    
        // Determine starting point based on cursor
        let start_index = if let Some(cursor_idx) = start_cursor {
            cursor_idx.min(total_count - 1)
        } else {
            0  // Always start at index 0, the direction will handle whether to go forward or backward
        };
    
        // Create paginated results from filtered results
        let mut paginated_results: Vec<SearchResult> = Vec::new();
        let mut end_index = start_index;  // Track where we ended for cursor calculation
        
        // Calculate page end index
        let end_bound = (start_index + page_size).min(total_count);
        
        // Get paginated slice
        paginated_results = filtered_results[start_index..end_bound].to_vec();
        
        end_index = end_bound - 1;
    
        // Calculate next cursor based on whether there are more results
        let next_cursor = if end_bound < total_count {
            Some(end_bound.to_string())
        } else {
            None  // No more results available
        };
    
        // Create response
        let response_data = SearchDriveResponseData {
            items: paginated_results.clone(),
            page_size: paginated_results.len(),
            total: total_count,
            direction,
            cursor: next_cursor,
        };
    
        create_response(
            StatusCode::OK,
            SearchDriveResponse::ok(&response_data).encode()
        )
    }


    pub async fn reindex_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Check if user is owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        // Get drive ID
        let drive_id = DRIVE_ID.with(|drive_id| drive_id.clone());
        
        // If not owner, check permissions
        if !is_owner {
            // Check if user has View permission on drive table or specific drive
            let table_resource_id = SystemResourceID::Table(SystemTableEnum::Drives);
            let specific_resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(drive_id.0.clone()));
            
            let table_permissions = check_system_permissions(
                table_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            let specific_permissions = check_system_permissions(
                specific_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            // User needs View permission on either the table or the specific drive
            let has_permission = table_permissions.contains(&SystemPermissionType::View) || 
                                specific_permissions.contains(&SystemPermissionType::View);
            
            if !has_permission {
                return create_auth_error_response();
            }
        }
    
        // Parse request body (optional)
        let body = request.body();
        let request_body: ReindexDriveRequestBody = if body.is_empty() {
            ReindexDriveRequestBody { force: None }
        } else {
            match serde_json::from_slice(body) {
                Ok(body) => body,
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid request format".to_string()).encode()
                ),
            }
        };

        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }
    
        // Check when the last reindex was performed
        let last_index_time = crate::core::state::search::state::state::get_last_index_update_time();
        let current_time = ic_cdk::api::time() / 1_000_000; // Convert nanoseconds to milliseconds
        
        // Only reindex if forced or if it's been at least 5 minutes since the last reindex
        let force = request_body.force.unwrap_or(false);
        if !force && last_index_time > 0 && (current_time - last_index_time) < 5 * 60 * 1000 {
            return create_response(
                StatusCode::TOO_MANY_REQUESTS,
                ErrorResponse::err(429, "Reindex was performed recently. Use 'force: true' to override.".to_string()).encode()
            );
        }
        
        // Perform the reindex
        let reindex_result = crate::core::state::search::state::state::reindex_drive();
        
        match reindex_result {
            Ok(indexed_count) => {
                // Get the updated timestamp
                let new_timestamp = crate::core::state::search::state::state::get_last_index_update_time();
                
                // Prepare response
                let response_data = ReindexDriveResponseData {
                    success: true,
                    timestamp_ms: new_timestamp,
                    indexed_count,
                };
                
                create_response(
                    StatusCode::OK,
                    ReindexDriveResponse::ok(&response_data).encode()
                )
            },
            Err(error) => {
                create_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::err(500, format!("Failed to reindex drive: {}", error)).encode()
                )
            }
        }
    }


    pub async fn external_id_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Check if user is owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        // If not owner, check for View permissions:
        // 1. On the entire Drives table, OR
        // 2. On the specific Drive_ID drive
        if !is_owner {
            let table_resource_id = SystemResourceID::Table(SystemTableEnum::Drives);
            let table_permissions = check_system_permissions(
                table_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            // Get the drive ID to check specific permissions
            let drive_id = DRIVE_ID.with(|drive_id| drive_id.clone());
            let drive_resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(drive_id.0.clone()));
            let drive_permissions = check_system_permissions(
                drive_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            // User needs View permission on either the table or the specific drive
            let has_permission = table_permissions.contains(&SystemPermissionType::View) || 
                               drive_permissions.contains(&SystemPermissionType::View);
            
            if !has_permission {
                return create_auth_error_response();
            }
        }
    
        // Parse request body
        let body = request.body();
        let request_body: ExternalIDsDriveRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }
    
        // If external_ids list is empty, just return an empty result list
        // This is a valid case that should return success with empty results
    
        // Process each external ID
        let mut results = Vec::new();
        
        EXTERNAL_ID_MAPPINGS.with(|mappings| {
            let mappings = mappings.borrow();
            
            for external_id in &request_body.external_ids {
                let result = if let Some(internal_ids) = mappings.get(external_id) {
                    ExternalIDvsInternalIDMaps {
                        success: true,
                        message: "External ID found".to_string(),
                        external_id: external_id.clone(),
                        internal_ids: internal_ids.items.clone(),
                    }
                } else {
                    ExternalIDvsInternalIDMaps {
                        success: false,
                        message: "External ID not found".to_string(),
                        external_id: external_id.clone(),
                        internal_ids: Vec::new(),
                    }
                };
                
                results.push(result);
            }
        });
    
        // Create response data
        let response_data = ExternalIDsDriveResponseData {
            results,
        };
    
        create_response(
            StatusCode::OK,
            ExternalIDsDriveResponse::ok(&response_data).encode()
        )
    }

    pub async fn transfer_ownership_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Take a snapshot early for audit/logging
        let prestate = snapshot_prestate();
    
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Verify that the requester is the current owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        if !is_owner {
            return create_response(
                StatusCode::UNAUTHORIZED,
                ErrorResponse::unauthorized().encode()
            );
        }
    
        // Parse request body
        let body: &[u8] = request.body();
        let transfer_request = match serde_json::from_slice::<TransferOwnershipDriveRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = transfer_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }
    
        // Validate that next_owner_id starts with the correct prefix
        let next_owner_id = transfer_request.next_owner_id;
        if !next_owner_id.starts_with(&IDPrefix::User.as_str()) {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid next_owner_id format. Must start with correct user prefix.".to_string()).encode()
            );
        }
    
        // Get current timestamp in milliseconds
        let current_timestamp_ms = ic_cdk::api::time() / 1_000_000;
        let one_day_ms: u64 = 24 * 60 * 60 * 1_000; // 24 hours in milliseconds
    
        let (status, ready_ms) = TRANSFER_OWNER_ID.with(|transfer_owner_id| {
            let current_transfer = transfer_owner_id.borrow().get().0.clone();
            
            // Check if there's an existing transfer request
            if !current_transfer.is_empty() {
                // Parse the existing transfer request
                let parts: Vec<&str> = current_transfer.split("::").collect();
                if parts.len() == 2 {
                    let existing_owner_id = parts[0];
                    if let Ok(transfer_timestamp_ms) = parts[1].parse::<u64>() {
                        // Check if the existing transfer is for the same owner and is older than 24 hours
                        if existing_owner_id == next_owner_id && current_timestamp_ms - transfer_timestamp_ms > one_day_ms {
                            // Complete the transfer
                            OWNER_ID.with(|owner_id| {
                                owner_id.borrow_mut().set(UserID(next_owner_id.clone()));
                            });
                            // Clear the transfer request
                            TRANSFER_OWNER_ID.with(|transfer_owner_id| {
                                transfer_owner_id.borrow_mut().set(UserID("".to_string()));
                            });
                            return (TransferOwnershipStatusEnum::Completed, current_timestamp_ms);
                        }
                    }
                }
            }
    
            // Set or update the transfer request
            let new_transfer_value = format!("{}::{}", next_owner_id, current_timestamp_ms);
            TRANSFER_OWNER_ID.with(|transfer_owner_id| {
                transfer_owner_id.borrow_mut().set(UserID(new_transfer_value));
            });
            
            // Calculate ready time in milliseconds
            let ready_time_ms = current_timestamp_ms + one_day_ms;
            (TransferOwnershipStatusEnum::Requested, ready_time_ms)
        });
    
        let response_data = TransferOwnershipResponseData {
            status,
            ready_ms,
        };
    
        // Log the transfer action
        let log_message = match response_data.status {
            TransferOwnershipStatusEnum::Completed => {
                format!(
                    "{}: Completed ownership transfer to {}", 
                    requester_api_key.user_id,
                    next_owner_id
                )
            },
            TransferOwnershipStatusEnum::Requested => {
                format!(
                    "{}: Initiated ownership transfer to {}", 
                    requester_api_key.user_id,
                    next_owner_id
                )
            }
        };
        
        snapshot_poststate(prestate, Some(log_message));
    
        create_response(
            StatusCode::OK,
            TransferOwnershipDriveResponse::ok(&response_data).encode()
        )
    }

    pub async fn whoami_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
    
        // Get organization ID from params
        let param_org_id = params.get("organization_id").unwrap().to_string();
        
        // If organization_id is "default", use the DRIVE_ID
        let organization_id = if param_org_id == "default" {
            DRIVE_ID.with(|id| id.clone())
        } else {
            DriveID(param_org_id)
        };
    
        // Get drive nickname from DRIVES_BY_ID_HASHTABLE
        let drive_nickname = DRIVES_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .get(&organization_id)
                .map(|drive| drive.name.clone())
                .unwrap_or_else(|| "".to_string())
        });
    
        // Get EVM public address from contacts
        let evm_public_address = CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .get(&requester_api_key.user_id)
                .map(|contact| contact.evm_public_address.clone())
                .unwrap_or_else(|| String::new())
        });
        
        // Get nickname from contacts
        let nickname = CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .get(&requester_api_key.user_id)
                .map(|contact| contact.name.clone())
                .unwrap_or_else(|| String::new())
        });
        
        // Extract the principal ID by removing "UserID_" prefix
        let user_id_str = requester_api_key.user_id.to_string();
        let icp_principal = if user_id_str.starts_with("UserID_") {
            user_id_str[7..].to_string()
        } else {
            user_id_str
        };
        
        let whoamireport = WhoAmIReport {
            nickname,
            userID: requester_api_key.user_id.clone(),
            driveID: organization_id,
            icp_principal: ICPPrincipalString(PublicKeyICP(icp_principal)),
            evm_public_address: Some(evm_public_address),
            is_owner,
            drive_nickname,
        };
    
        create_response(
            StatusCode::OK,
            GetWhoAmIResponse::ok(&whoamireport).encode()
        )
    }

    pub async fn superswap_userid_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Check if user is owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        match is_owner {
            true => {
                // Parse request body
                let body = request.body();
                let request_body: SuperswapUserIDRequestBody = match serde_json::from_slice(body) {
                    Ok(body) => body,
                    Err(_) => return create_response(
                        StatusCode::BAD_REQUEST,
                        ErrorResponse::err(400, "Invalid request format".to_string()).encode()
                    ),
                };
    
                if let Err(validation_error) = request_body.validate_body() {
                    return create_response(
                        StatusCode::BAD_REQUEST,
                        ErrorResponse::err(400, format!("Validation error: {}: {}", 
                            validation_error.field, validation_error.message)).encode()
                    );
                }

                // snapshot prestate
                let prestate = snapshot_prestate();
    
                match superswap_userid(
                    UserID(request_body.current_user_id.clone()),
                    UserID(request_body.new_user_id.clone())
                ) {
                    Ok(update_count) => {

                        // check webhooks and fire
                        let active_webhooks = get_superswap_user_webhooks(
                            WebhookEventLabel::OrganizationSuperswapUser
                        );

                        fire_superswap_user_webhook(
                            WebhookEventLabel::OrganizationSuperswapUser,
                            active_webhooks,
                            Some(UserID(request_body.current_user_id.clone())),
                            Some(UserID(request_body.new_user_id.clone())),
                            Some(format!("'{}' superswapped to '{}', updated {} records", 
                                request_body.current_user_id, request_body.new_user_id, update_count))
                        );

                        // snapshot poststate
                        snapshot_poststate(prestate, Some(format!("'{}' superswapped to '{}'", 
                            request_body.current_user_id, request_body.new_user_id)));

                        // rest response
                        let response_data = SuperswapUserIDResponseData {
                            success: true,
                            message: format!("'{}' superswapped to '{}', updated {} records", 
                                request_body.current_user_id, request_body.new_user_id, update_count), 
                        };
                        create_response(
                            StatusCode::OK,
                            SuperswapUserIDResponse::ok(&response_data).encode()
                        )
                    },
                    Err(_) => {
                        create_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ErrorResponse::err(500, "Failed to superswap user ID".to_string()).encode()
                        )
                    }
                }
            },
            false => {
                create_response(
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse::unauthorized().encode()
                )
            }
        }
    }

    pub async fn redeem_organization_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        
    
        // Parse the request body
        let body = request.body();
        let request_body: RedeemOrgRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // Validate the request body
        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }
    
        // Check if redeem code exists and hasn't been redeemed yet
        let stored_redeem_code = SPAWN_REDEEM_CODE.with(|code| code.borrow().get().0.clone());
        
        // Check if the code has already been redeemed (empty string)
        if stored_redeem_code.is_empty() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Spawn code has already been redeemed".to_string()).encode()
            );
        }
        
        // Check if the provided code matches the stored code
        if request_body.redeem_code != stored_redeem_code {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid redeem code".to_string()).encode()
            );
        }
    
        // Get the necessary drive data
        let drive_id = DRIVE_ID.with(|id| id.clone());
        let canister_id = CANISTER_ID.with(|id| id.0.clone());
        let endpoint_url = URL_ENDPOINT.with(|url| url.borrow().get().0.clone());
        let spawn_note = SPAWN_NOTE.with(|note| note.borrow().get().clone());
        
        // Get the owner's default admin API key
        let owner_id = OWNER_ID.with(|id| id.borrow().get().clone());
        let mut admin_api_key = String::new();
        
        crate::core::state::api_keys::state::state::USERS_APIKEYS_HASHTABLE.with(|map| {
            if let Some(api_key_ids) = map.borrow().get(&owner_id) {
                if !api_key_ids.is_empty() {
                    crate::core::state::api_keys::state::state::APIKEYS_BY_ID_HASHTABLE.with(|id_map| {
                        if let Some(first_key_id) = api_key_ids.keys.first() {
                            if let Some(api_key) = id_map.borrow().get(first_key_id) {
                                admin_api_key = api_key.value.0.clone();
                            }
                        }
                    });
                }
            }
        });
    
        // Construct the admin login password
        let admin_login_password = format!("{}:{}@{}", drive_id, admin_api_key, endpoint_url);
    
        // Create the response data
        let response_data = RedeemOrgResponseData {
            drive_id,
            endpoint_url: endpoint_url,
            api_key: admin_api_key,
            note: spawn_note,
            admin_login_password,
        };
    
        // Reset the redemption code to empty string (mark as redeemed)
        SPAWN_REDEEM_CODE.with(|code| {
            code.borrow_mut().set(SpawnRedeemCode("".to_string()));
            debug_log!("Spawn redeem code has been used and reset");
        });
        
    
        // Encode and return the response
        create_response(
            StatusCode::OK,
            RedeemOrgResponse::ok(&response_data).encode()
        )
    }

    pub async fn inbox_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body = request.body();
        let request_body: InboxOrgRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }

    
        // Check if user is owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        // If not owner, check for View permissions:
        // 1. On the entire Drives table, OR
        // 2. On the specific Drive_ID drive
        if !is_owner {
            let table_resource_id = SystemResourceID::Table(SystemTableEnum::Inbox);
            let table_permissions = check_system_permissions(
                table_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            
            // User needs View permission on either the table or the specific drive
            let has_permission = table_permissions.contains(&SystemPermissionType::Create);
            
            if !has_permission {
                return create_auth_error_response();
            }
        }

        let inbox_notif_id = InboxNotifID(generate_uuidv4(IDPrefix::InboxNotifID));
        let timestamp_ms = ic_cdk::api::time() / 1_000_000;
    

        let active_webhooks = get_org_inbox_webhooks(request_body.topic.as_ref());

        debug_log!("Active webhooks: {:?}", active_webhooks);

        fire_org_inbox_new_notif_webhook(
            WebhookEventLabel::OrganizationInboxNewNotif,
            active_webhooks,
            None,
            Some(request_body.clone()),
            Some("New inbox notification received".to_string())
        ).await;
    
        // Create response data
        let response_data = InboxOrgResponseData {
            inbox_notif_id,
            drive_id: DRIVE_ID.with(|id| id.clone()),
            timestamp_ms,
            note: "Inbox notification received".to_string(),
        };

        create_response(
            StatusCode::OK,
            InboxOrgResponse::ok(&response_data).encode()
        )
    }


    fn json_decode<T>(value: &[u8]) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_slice(value).expect("Failed to deserialize value")
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