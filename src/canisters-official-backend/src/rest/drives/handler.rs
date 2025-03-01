// src/rest/drives/handler.rs


pub mod drives_handlers {
    use crate::{
        core::{api::{permissions::{directory::{can_user_access_directory_permission, check_directory_permissions}, system::{can_user_access_system_permission, check_system_permissions}}, replay::diff::{apply_state_diff, safely_apply_diffs, snapshot_entire_state, snapshot_poststate, snapshot_prestate}, uuid::generate_unique_id}, state::{api_keys::state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, disks::state::state::{DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, drives::{state::state::{update_external_id_mapping, DRIVES_BY_ID_HASHTABLE, DRIVES_BY_TIME_LIST, DRIVE_ID, DRIVE_STATE_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, EXTERNAL_ID_MAPPINGS, OWNER_ID, TRANSFER_OWNER_ID, URL_ENDPOINT}, types::{Drive, DriveID, DriveRESTUrlEndpoint, DriveStateDiffID, ExternalID, ExternalPayload}}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE}, types::{DirectoryPermissionType, PermissionGranteeID, SystemPermissionType, SystemResourceID, SystemTableEnum}}, search::types::SearchCategoryEnum, tags::{state::{add_tag_to_resource, parse_tag_resource_id, remove_tag_from_resource, validate_tag_value}, types::{TagOperationResponse, TagResourceID}}, team_invites::state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, teams::state::state::{is_team_admin, TEAMS_BY_ID_HASHTABLE, TEAMS_BY_TIME_LIST}}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID, EXTERNAL_PAYLOAD_MAX_LEN}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::DirectoryResourceID, drives::types::{CreateDriveResponse, DeleteDriveRequest, DeleteDriveResponse, DeletedDriveData, ErrorResponse, ExternalIDsDriveRequestBody, ExternalIDsDriveResponse, ExternalIDsDriveResponseData, ExternalIDvsInternalIDMaps, GetDriveResponse, ListDrivesRequestBody, ListDrivesResponse, ListDrivesResponseData, ReindexDriveRequestBody, ReindexDriveResponse, ReindexDriveResponseData, ReplayDriveRequestBody, ReplayDriveResponse, ReplayDriveResponseData, SearchDriveRequestBody, SearchDriveResponse, SearchDriveResponseData, TransferOwnershipDriveRequestBody, TransferOwnershipDriveResponse, TransferOwnershipResponseData, TransferOwnershipStatusEnum, UpdateDriveResponse, UpsertDriveRequestBody}, webhooks::types::SortDirection}
        
    };
    use serde_json::json;
    use crate::core::state::search::state::state::{raw_query,filter_search_results_by_permission};
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Get drive ID from params
        let drive_id = DriveID(params.get("drive_id").unwrap().to_string());

        // Get the drive
        let drive = DRIVES_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&drive_id).cloned()
        });

        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Drives),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(drive_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) || !table_permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }
    

        match drive {
            Some(mut drive) => {
                if !is_owner {
                    drive.private_note = None;
                }
                create_response(
                    StatusCode::OK,
                    GetDriveResponse::ok(&drive).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_drives_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access drives
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Drives);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        // Parse request body
        let body = request.body();
        let request_body: ListDrivesRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

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

        // Get total count
        let total_count = DRIVES_BY_TIME_LIST.with(|list| list.borrow().len());

        // If there are no drives, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListDrivesResponse::ok(&ListDrivesResponseData {
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

        // Get drives with pagination and filtering
        let mut filtered_drives = Vec::new();
        let mut processed_count = 0;

        DRIVES_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            DRIVES_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        // Newest first
                        let mut current_idx = start_index;
                        while filtered_drives.len() < request_body.page_size && current_idx < total_count {
                            if let Some(drive) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() {
                                    filtered_drives.push(drive.clone());
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
                        // Oldest first
                        let mut current_idx = start_index;
                        while filtered_drives.len() < request_body.page_size && current_idx < total_count {
                            if let Some(drive) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() {
                                    filtered_drives.push(drive.clone());
                                }
                            }
                            current_idx += 1;
                            processed_count = current_idx - start_index;
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
                let next_up = if processed_count > 0 {
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

        // Create response
        let response_data = ListDrivesResponseData {
            items: filtered_drives.clone(),
            page_size: filtered_drives.len(),
            total: total_count,
            cursor_up,
            cursor_down,
        };

        create_response(
            StatusCode::OK,
            ListDrivesResponse::ok(&response_data).encode()
        )
    }

    pub async fn upsert_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body: &[u8] = request.body();

        if let Ok(req) = serde_json::from_slice::<UpsertDriveRequestBody>(body) {
            match req {
                UpsertDriveRequestBody::Update(update_req) => {
                    let drive_id = DriveID(update_req.id);
                    
                    // Get existing drive
                    let mut drive = match DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().get(&drive_id).cloned()) {
                        Some(drive) => drive,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };

                    if !is_owner {
                        let table_permissions = check_system_permissions(
                            SystemResourceID::Table(SystemTableEnum::Drives),
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        let resource_id = SystemResourceID::Record(drive_id.to_string());
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Update) && !table_permissions.contains(&SystemPermissionType::Update) {
                            return create_auth_error_response();
                        }
                    }

                    let prestate = snapshot_prestate();

                    // Update fields
                    if let Some(name) = update_req.name {
                        drive.name = name;
                    }
                    if let Some(public_note) = update_req.public_note {
                        drive.public_note = Some(public_note);
                    }
                    if let Some(private_note) = update_req.private_note {
                        drive.private_note = Some(private_note);
                    }
                    if let Some(icp_principal) = update_req.icp_principal {
                        drive.icp_principal = ICPPrincipalString(PublicKeyICP(icp_principal));
                    }
                    if let Some(url_endpoint) = update_req.url_endpoint {
                        drive.url_endpoint = DriveRESTUrlEndpoint(url_endpoint.trim_end_matches('/')
                        .to_string());
                    }

                    if let Some(external_id) = update_req.external_id.clone() {
                        let old_external_id = drive.external_id.clone();
                        let new_external_id = Some(ExternalID(external_id.clone()));
                        drive.external_id = new_external_id.clone();
                        update_external_id_mapping(
                            old_external_id,
                            new_external_id,
                            Some(drive.id.to_string())
                        );
                    }
                    if let Some(external_payload) = update_req.external_payload.clone() {
                        // Check length of external_payload (limit: 8192 characters)
                        if external_payload.len() > EXTERNAL_PAYLOAD_MAX_LEN {
                            return create_response(
                                StatusCode::BAD_REQUEST,
                                ErrorResponse::err(
                                    400, 
                                    format!(
                                        "external_payload is too large ({} bytes). Max allowed is {} chars",
                                        external_payload.len(),
                                        EXTERNAL_PAYLOAD_MAX_LEN
                                    )
                                ).encode()
                            );
                        }
                        drive.external_payload = Some(ExternalPayload(external_payload));
                    }

                    DRIVES_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(drive_id.clone(), drive.clone());
                    });

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Update Drive {}", 
                            requester_api_key.user_id,
                            drive_id.clone()
                        ).to_string()
                    ));

                    create_response(
                        StatusCode::OK,
                        UpdateDriveResponse::ok(&drive).encode()
                    )
                },
                UpsertDriveRequestBody::Create(create_req) => {
                    if !is_owner {
                        let resource_id = SystemResourceID::Table(SystemTableEnum::Drives);
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Create) {
                            return create_auth_error_response();
                        }
                    }
                    let prestate = snapshot_prestate();

                    if let Some(external_payload) = create_req.external_payload.clone() {
                        // Check length of external_payload (limit: 8192 characters)
                        if external_payload.len() > EXTERNAL_PAYLOAD_MAX_LEN {
                            return create_response(
                                StatusCode::BAD_REQUEST,
                                ErrorResponse::err(
                                    400, 
                                    format!(
                                        "external_payload is too large ({} bytes). Max allowed is {} chars",
                                        external_payload.len(),
                                        EXTERNAL_PAYLOAD_MAX_LEN
                                    )
                                ).encode()
                            );
                        }
                    }

                    // Create new drive
                    let drive_id = DriveID(generate_unique_id(IDPrefix::Drive, ""));
                    let drive = Drive {
                        id: drive_id.clone(),
                        name: create_req.name,
                        public_note: Some(create_req.public_note.unwrap_or_default()),
                        private_note: Some(create_req.private_note.unwrap_or_default()),
                        icp_principal: ICPPrincipalString(PublicKeyICP(create_req.icp_principal.unwrap_or_default())),
                        url_endpoint: DriveRESTUrlEndpoint(
                            create_req.url_endpoint
                                .unwrap_or(URL_ENDPOINT.with(|url| url.borrow().clone()).0)
                                .trim_end_matches('/')
                                .to_string()
                        ),
                        last_indexed_ms: None,
                        tags: vec![],
                        external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
                        external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
                    };

                    DRIVES_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(drive_id.clone(), drive.clone());
                    });

                    DRIVES_BY_TIME_LIST.with(|store| {
                        store.borrow_mut().push(drive_id.clone());
                    });

                    update_external_id_mapping(
                        None,
                        Some(drive.external_id.clone().unwrap()),
                        Some(drive_id.to_string())
                    );

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Create Drive {}", 
                            requester_api_key.user_id,
                            drive_id.clone()
                        ).to_string()
                    ));

                    create_response(
                        StatusCode::OK,
                        CreateDriveResponse::ok(&drive).encode()
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

    pub async fn delete_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteDriveRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        let drive_id = delete_request.id;
        let drive = match DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().get(&drive_id).cloned()) {
            Some(drive) => drive,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = drive.external_id.clone();
        let old_internal_id = Some(drive_id.to_string());

        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Drives),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(drive_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }
        let prestate = snapshot_prestate();

        // Remove from hashtable
        DRIVES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&drive_id);
        });

        // Remove from time list
        DRIVES_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &drive_id);
        });

        update_external_id_mapping(old_external_id, None, old_internal_id);

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Drive {}", 
                requester_api_key.user_id,
                drive_id.clone()
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            DeleteDriveResponse::ok(&DeletedDriveData {
                id: drive_id,
                deleted: true
            }).encode()
        )
    }

    pub async fn snapshot_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // temporarily disabled for testing
        // let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        // if !is_owner {
        //     return create_auth_error_response();
        // }

        debug_log!("Requester API Key, {:?}", requester_api_key);

        let snapshot = snapshot_entire_state();

        // Return the JSON response
        match serde_json::to_vec(&snapshot) {
            Ok(json) => create_response(StatusCode::OK, json),
            Err(_) => create_response(
                StatusCode::INTERNAL_SERVER_ERROR, 
                ErrorResponse::err(500, "Failed to serialize state".to_string()).encode()
            )
        }
    }

    pub async fn replay_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
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
                    let current_timestamp = DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get());
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
                    timestamp_ns: DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get()),
                    diffs_applied: applied_count,
                    checkpoint_diff_id: last_diff_id,
                    final_checksum: DRIVE_STATE_CHECKSUM.with(|cs| cs.borrow().clone()),
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        // Parse request body
        let body = request.body();
        let request_body: SearchDriveRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // Check if search query is provided
        if request_body.query.trim().is_empty() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Search query cannot be empty".to_string()).encode()
            );
        }
    
        // Use the categories from the request or default to All if empty
        let categories = if request_body.categories.is_empty() {
            Some(vec![SearchCategoryEnum::All])
        } else {
            Some(request_body.categories)
        };
    
        // Perform the search using the search module
        let max_edit_distance = 2; // Allow up to 2 character edits for fuzzy matching
        let search_results = raw_query(&request_body.query, max_edit_distance, categories);
        
        // Create a PermissionGranteeID from the requester's user ID for permission checks
        let grantee_id = PermissionGranteeID::User(requester_api_key.user_id.clone());
        
        // Filter results based on permissions
        let filtered_results = filter_search_results_by_permission(&search_results, &grantee_id, is_owner).await;
        
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
                    cursor_up: None,
                    cursor_down: None,
                }).encode()
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
    
        // Create paginated results from filtered results
        let mut paginated_results = Vec::new();
        let mut processed_count = 0;
        
        match request_body.direction {
            SortDirection::Desc => {
                // Newest first (highest index to lowest)
                let mut current_idx = start_index;
                while paginated_results.len() < request_body.page_size && current_idx < total_count {
                    paginated_results.push(filtered_results[current_idx].clone());
                    if current_idx == 0 {
                        break;
                    }
                    current_idx -= 1;
                    processed_count = start_index - current_idx;
                }
            },
            SortDirection::Asc => {
                // Oldest first (lowest index to highest)
                let mut current_idx = start_index;
                while paginated_results.len() < request_body.page_size && current_idx < total_count {
                    paginated_results.push(filtered_results[current_idx].clone());
                    current_idx += 1;
                    if current_idx >= total_count {
                        break;
                    }
                    processed_count = current_idx - start_index;
                }
            }
        }
    
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
    
        // Create response
        let response_data = SearchDriveResponseData {
            items: paginated_results.clone(),
            page_size: paginated_results.len(),
            total: total_count,
            cursor_up,
            cursor_down,
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        // Get drive ID
        let drive_id = DRIVE_ID.with(|drive_id| drive_id.clone());
        
        // If not owner, check permissions
        if !is_owner {
            // Check if user has View permission on drive table or specific drive
            let table_resource_id = SystemResourceID::Table(SystemTableEnum::Drives);
            let specific_resource_id = SystemResourceID::Record(drive_id.0.clone());
            
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
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
            let drive_resource_id = SystemResourceID::Record(drive_id.0.clone());
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
                        internal_ids: internal_ids.clone(),
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
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
            let current_transfer = transfer_owner_id.borrow().0.clone();
            
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
                                *owner_id.borrow_mut() = UserID(next_owner_id.clone());
                            });
                            // Clear the transfer request
                            *transfer_owner_id.borrow_mut() = UserID("".to_string());
                            return (TransferOwnershipStatusEnum::Completed, current_timestamp_ms);
                        }
                    }
                }
            }
    
            // Set or update the transfer request
            let new_transfer_value = format!("{}::{}", next_owner_id, current_timestamp_ms);
            *transfer_owner_id.borrow_mut() = UserID(new_transfer_value);
            
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