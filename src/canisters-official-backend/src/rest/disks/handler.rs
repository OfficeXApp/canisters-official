// src/rest/disks/handler.rs


pub mod disks_handlers {
    use crate::{
        core::{api::{internals::drive_internals::validate_auth_json, permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::generate_unique_id}, state::{disks::{state::state::{ensure_disk_root_folder, DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, types::{AwsBucketAuth, Disk, DiskID, DiskTypeEnum}}, drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemResourceID, SystemTableEnum}}, types::{IDPrefix, EXTERNAL_PAYLOAD_MAX_LEN}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, disks::types::{ CreateDiskResponse, DeleteDiskRequest, DeleteDiskResponse, DeletedDiskData, ErrorResponse, GetDiskResponse, ListDisksRequestBody, ListDisksResponse, ListDisksResponseData, UpdateDiskResponse, UpsertDiskRequestBody}, webhooks::types::SortDirection}
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access disk.private_note
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Get disk ID from params
        let disk_id = DiskID(params.get("disk_id").unwrap().to_string());

        // Get the disk
        let disk = DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&disk_id).cloned()
        });

        // Check permissions if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Disks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(disk_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        match disk {
            Some(mut disk) => {
                if !is_owner {
                    disk.private_note = None;
                    disk.auth_json = None;
                }
                create_response(
                    StatusCode::OK,
                    GetDiskResponse::ok(&disk).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_disks_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Check table permissions if not owner
        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Disks);
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
        let request_body: ListDisksRequestBody = match serde_json::from_slice(body) {
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
        let total_count = DISKS_BY_TIME_LIST.with(|list| list.borrow().len());

        // If there are no disks, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListDisksResponse::ok(&ListDisksResponseData {
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

        // Get disks with pagination and filtering
        let mut filtered_disks = Vec::new();
        let mut processed_count = 0;

        DISKS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            DISKS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_disks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(disk) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() {
                                    filtered_disks.push(disk.clone());
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
                        while filtered_disks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(disk) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() {
                                    filtered_disks.push(disk.clone());
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

        create_response(
            StatusCode::OK,
            ListDisksResponse::ok(&ListDisksResponseData {
                items: filtered_disks.clone(),
                page_size: filtered_disks.len(),
                total: total_count,
                cursor_up,
                cursor_down,
            }).encode()
        )
    }


    pub async fn upsert_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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

        if let Ok(req) = serde_json::from_slice::<UpsertDiskRequestBody>(body) {
            match req {
                UpsertDiskRequestBody::Update(update_req) => {
                    let disk_id = DiskID(update_req.id);
                    
                    // Get existing disk
                    let mut disk = match DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&disk_id).cloned()) {
                        Some(disk) => disk,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };

                    // Check update permission if not owner
                    if !is_owner {
                        let table_permissions = check_system_permissions(
                            SystemResourceID::Table(SystemTableEnum::Disks),
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        let resource_id = SystemResourceID::Record(disk_id.to_string());
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
                    if let Some(private_note) = update_req.private_note {
                        disk.private_note = Some(private_note);
                    }
                    if let Some(auth_json) = update_req.auth_json {
                        // Validate auth_json if provided
                        if let Err(err_msg) = validate_auth_json(&disk.disk_type, &Some(auth_json.clone())) {
                            return create_response(
                                StatusCode::BAD_REQUEST,
                                ErrorResponse::err(400, err_msg).encode()
                            );
                        }
                        disk.auth_json = Some(auth_json);
                    }
                    if let Some(name) = update_req.name {
                        disk.name = name;
                    }
                    if let Some(public_note) = update_req.public_note {
                        disk.public_note = Some(public_note);
                    }
                    if let Some(external_id) = update_req.external_id {
                        let old_external_id = disk.external_id.clone();
                        let new_external_id = Some(ExternalID(external_id));
                        disk.external_id = new_external_id.clone();
                        update_external_id_mapping(
                            old_external_id,
                            new_external_id,
                            Some(disk_id.0.clone())
                        );
                    }
                    if let Some(external_payload) = update_req.external_payload {
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
                        disk.external_payload = Some(ExternalPayload(external_payload));
                    }

                    DISKS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(disk_id.clone(), disk.clone());
                    });

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Update Disk {}", 
                            requester_api_key.user_id,
                            disk_id.clone()
                        ).to_string())
                    );

                    create_response(
                        StatusCode::OK,
                        UpdateDiskResponse::ok(&disk).encode()
                    )
                },
                UpsertDiskRequestBody::Create(create_req) => {

                    // Check create permission if not owner
                    if !is_owner {
                        let resource_id = SystemResourceID::Table(SystemTableEnum::Disks);
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Create) {
                            return create_auth_error_response();
                        }
                    }
                    
                    // Validate that auth_json is provided and valid for AwsBucket or StorjWeb3 types.
                    if let Err(err_msg) = validate_auth_json(&create_req.disk_type, &create_req.auth_json) {
                        return create_response(
                            StatusCode::BAD_REQUEST,
                            ErrorResponse::err(400, err_msg).encode()
                        );
                    }
                    let prestate = snapshot_prestate();

                    // Check external_payload size before creating
                    if let Some(ref external_payload) = create_req.external_payload {
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
                    
                    // Create new disk
                    let disk_type_suffix = format!("__DiskType_{}", create_req.disk_type);
                    let disk_id = DiskID(generate_unique_id(IDPrefix::Disk, &disk_type_suffix));
                    let new_external_id = Some(ExternalID(create_req.external_id.unwrap_or("".to_string())));
                    let disk = Disk {
                        id: disk_id.clone(),
                        name: create_req.name,
                        public_note: create_req.public_note,
                        private_note: create_req.private_note,
                        auth_json: create_req.auth_json,
                        disk_type: create_req.disk_type,
                        tags: vec![],
                        external_id: new_external_id.clone(),
                        external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
                    };
                    update_external_id_mapping(
                        None,
                        new_external_id,
                        Some(disk_id.0.clone())
                    );

                    // Store the disk
                    DISKS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(disk_id.clone(), disk.clone());
                    });

                    DISKS_BY_TIME_LIST.with(|store| {
                        store.borrow_mut().push(disk_id.clone());
                    });

                    ensure_disk_root_folder(
                        &disk_id,
                        &requester_api_key.user_id,
                        &ic_cdk::api::id().to_text()
                    );

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Create Disk {}", 
                            requester_api_key.user_id,
                            disk_id.clone()
                        ).to_string())
                    );

                    create_response(
                        StatusCode::OK,
                        CreateDiskResponse::ok(&disk).encode()
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

    pub async fn delete_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        let prestate = snapshot_prestate();

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteDiskRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        let disk_id = delete_request.id.clone();

        // Check delete permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Disks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(disk_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) || !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        // Get disk for external ID cleanup
        let disk = DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&disk_id).cloned()
        });

        // Remove from main stores
        DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&disk_id);
        });

        DISKS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &disk_id);
        });

        // Remove from external ID mappings
        if let Some(disk) = disk {
            update_external_id_mapping(
                disk.external_id,
                None,
                None,
            );
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Disk {}", 
                requester_api_key.user_id,
                disk_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            DeleteDiskResponse::ok(&DeletedDiskData {
                id: disk_id,
                deleted: true
            }).encode()
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