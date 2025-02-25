// src/rest/drives/handler.rs


pub mod drives_handlers {
    use crate::{
        core::{api::{permissions::system::check_system_permissions, replay::diff::{apply_state_diff, snapshot_poststate, snapshot_prestate}, uuid::generate_unique_id}, state::{api_keys::state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, disks::state::state::{DISKS_BY_EXTERNAL_ID_HASHTABLE, DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, drives::{state::state::{DRIVES_BY_ID_HASHTABLE, DRIVES_BY_TIME_LIST, DRIVE_STATE_TIMESTAMP_NS, OWNER_ID, URL_ENDPOINT}, types::{Drive, DriveID, DriveRESTUrlEndpoint, DriveStateDiffID}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemResourceID, SystemTableEnum}, team_invites::state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, teams::state::state::{TEAMS_BY_ID_HASHTABLE, TEAMS_BY_TIME_LIST}}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, drives::types::{CreateDriveResponse, DeleteDriveRequest, DeleteDriveResponse, DeletedDriveData, ErrorResponse, GetDriveResponse, ListDrivesRequestBody, ListDrivesResponse, ListDrivesResponseData, ReplayDriveRequestBody, ReplayDriveResponse, ReplayDriveResponseData, UpdateDriveResponse, UpsertDriveRequestBody}, webhooks::types::SortDirection}
        
    };
    use serde_json::json;
    use crate::core::state::drives::{
        
    };
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
            let resource_id = SystemResourceID::Record(drive_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) {
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
                        let resource_id = SystemResourceID::Record(drive_id.to_string());
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Update) {
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
                    };

                    DRIVES_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(drive_id.clone(), drive.clone());
                    });

                    DRIVES_BY_TIME_LIST.with(|store| {
                        store.borrow_mut().push(drive_id.clone());
                    });

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

        if !is_owner {
            let resource_id = SystemResourceID::Record(drive_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) {
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
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            return create_auth_error_response();
        }
    
        // Collect all state data using serde_json::json! macro
        let prestate = snapshot_prestate();
    
        // Return the JSON response
        match serde_json::to_vec(&prestate) {
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
        
        // Take a snapshot of the state before making changes, for potential audit or logging
        let prestate = snapshot_prestate();
        
        // Apply each diff in order
        let mut applied_count = 0;
        let mut last_diff_id: Option<DriveStateDiffID> = None;
        let mut last_diff_timestamp_ns: Option<u64> = None;
        
        for diff_data in replay_request.diffs.iter() {
            match apply_state_diff(&diff_data.diff) {
                Ok(_) => {
                    applied_count += 1;
                    last_diff_id = Some(diff_data.id.clone());
                    last_diff_timestamp_ns = Some(diff_data.timestamp_ns);
                    debug_log!("Applied diff: {}", diff_data.id);
                },
                Err(e) => {
                    debug_log!("Error applying diff {}: {}", diff_data.id, e);
                    // Continue with next diff even if one fails
                }
            }
        }

        // Log notes if provided
        let notes_str = format!(
            "{}: Replay diffs to {} - {}", 
            requester_api_key.user_id,
            last_diff_timestamp_ns.clone().unwrap_or_default(),
            replay_request.notes.clone().unwrap_or_default()
        );
        debug_log!("Replay operation notes: {}", notes_str);
    
        
        // If any diffs were applied, finalize by recording the operation
        if applied_count > 0 {
            snapshot_poststate(prestate, Some(notes_str));
        }
        
        // Prepare response data
        let response_data = ReplayDriveResponseData {
            timestamp_ns: DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get()) / 1_000_000,
            diffs_applied: applied_count,
            checkpoint_diff_id: last_diff_id,
        };
    
        create_response(
            StatusCode::OK,
            ReplayDriveResponse::ok(&response_data).encode()
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