// src/rest/disks/handler.rs


pub mod disks_handlers {
    use crate::{
        core::{api::{internals::drive_internals::validate_auth_json, permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}}, state::{disks::{state::state::{ensure_disk_root_and_trash_folder, DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, types::{AwsBucketAuth, Disk, DiskID, DiskTypeEnum}}, drives::{state::state::{update_external_id_mapping, DRIVE_ID, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::IDPrefix}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, disks::types::{ CreateDiskRequestBody, CreateDiskResponse, DeleteDiskRequest, DeleteDiskResponse, DeletedDiskData, ErrorResponse, GetDiskResponse, ListDisksRequestBody, ListDisksResponse, ListDisksResponseData, UpdateDiskRequestBody, UpdateDiskResponse}, webhooks::types::SortDirection}
        
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
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.to_string()));
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
                create_response(
                    StatusCode::OK,
                    GetDiskResponse::ok(&disk.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_disks_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
    
        debug_log!("Handling list_disks_handler...");
        
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Check if the requester is the owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
    
        // Check table-level permissions if not owner
        let has_table_permission = if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Disks);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            permissions.contains(&SystemPermissionType::View)
        } else {
            true
        };

        debug_log!("has_table_permission: {}", has_table_permission);
    
        // Parse request body
        let body = request.body();
        let request_body: ListDisksRequestBody = match serde_json::from_slice(body) {
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
    
        // Parse cursor if provided
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
                    direction: request_body.direction,
                    cursor: None,
                }).encode()
            );
        }
    
        // Determine starting point based on cursor
        let start_index = if let Some(cursor_idx) = start_cursor {
            cursor_idx.min(total_count - 1)
        } else {
            match request_body.direction {
                SortDirection::Asc => 0,
                SortDirection::Desc => total_count - 1,
            }
        };
    
        // Get disks with pagination and filtering, applying permission checks
        let mut filtered_disks = Vec::new();
        let mut processed_count = 0;
        let mut end_index = start_index;  // Track where we ended for cursor calculation
        let mut total_count_to_return = 0; // Will use this for the response
    
        // If user is owner or has table access, they get the actual total count
        if is_owner || has_table_permission {
            total_count_to_return = total_count;
        }
    
        DISKS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            DISKS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_disks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(disk) = id_store.get(&time_index[current_idx]) {
                                // Check if user has permission to view this specific disk
                                let can_view = is_owner || has_table_permission || {
                                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk.id.to_string()));
                                    let permissions = check_system_permissions(
                                        resource_id,
                                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                                    );
                                    permissions.contains(&SystemPermissionType::View)
                                };
    
                                if can_view && request_body.filters.is_empty() {
                                    filtered_disks.push(disk.clone());
                                }
                            }
                            if current_idx == 0 {
                                break;
                            }
                            current_idx -= 1;
                            processed_count += 1;
                        }
                        end_index = current_idx;
                    },
                    SortDirection::Asc => {
                        let mut current_idx = start_index;
                        while filtered_disks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(disk) = id_store.get(&time_index[current_idx]) {
                                // Check if user has permission to view this specific disk
                                let can_view = is_owner || has_table_permission || {
                                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk.id.to_string()));
                                    let permissions = check_system_permissions(
                                        resource_id,
                                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                                    );
                                    permissions.contains(&SystemPermissionType::View)
                                };
    
                                if can_view && request_body.filters.is_empty() {
                                    filtered_disks.push(disk.clone());
                                }
                            }
                            current_idx += 1;
                            processed_count += 1;
                            if current_idx >= total_count {
                                break;
                            }
                        }
                        end_index = current_idx - 1;
                    }
                }
            });
        });
    
        // Calculate next cursor based on direction and where we ended
        let next_cursor = if filtered_disks.len() >= request_body.page_size {
            match request_body.direction {
                SortDirection::Desc => {
                    if end_index > 0 {
                        Some(end_index.to_string())
                    } else {
                        None
                    }
                },
                SortDirection::Asc => {
                    if end_index < total_count - 1 {
                        Some((end_index + 1).to_string())
                    } else {
                        None
                    }
                }
            }
        } else {
            None  // No more results available
        };
    
        // Determine the total count for the response
        // If the user doesn't have full access and we haven't calculated the total yet,
        // set it to batch size + 1 if there are more results available
        if !is_owner && !has_table_permission {
            if next_cursor.is_some() {
                // If there are more results (next cursor exists), return batch size + 1
                total_count_to_return = filtered_disks.len() + 1;
            } else {
                // Otherwise, just return the batch size
                total_count_to_return = filtered_disks.len();
            }
        }
    
        create_response(
            StatusCode::OK,
            ListDisksResponse::ok(&ListDisksResponseData {
                items: filtered_disks.clone().into_iter().map(|disk| {
                    disk.cast_fe(&requester_api_key.user_id)
                }).collect(),
                page_size: filtered_disks.len(),
                total: total_count_to_return,
                direction: request_body.direction,
                cursor: next_cursor,
            }).encode()
        )
    }

    pub async fn create_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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
        let create_req = serde_json::from_slice::<CreateDiskRequestBody>(body).unwrap();
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

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

        
        // Create new disk
        let disk_id = match create_req.id {
            Some(id) => DiskID(id.to_string()),
            None => DiskID(generate_uuidv4(IDPrefix::Disk)),
        };

        let (root_folder_uuid, trash_folder_uuid) = ensure_disk_root_and_trash_folder(
            &disk_id,
            &requester_api_key.user_id,
            &DRIVE_ID.with(|drive_id| drive_id.clone()),
            create_req.disk_type.clone()
        );

        let new_external_id = Some(ExternalID(create_req.external_id.unwrap_or("".to_string())));
        let disk = Disk {
            id: disk_id.clone(),
            name: create_req.name,
            public_note: create_req.public_note,
            private_note: create_req.private_note,
            auth_json: create_req.auth_json,
            disk_type: create_req.disk_type,
            labels: vec![],
            created_at: ic_cdk::api::time() / 1_000_000,
            root_folder: root_folder_uuid,
            trash_folder: trash_folder_uuid,
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
        mark_claimed_uuid(&disk_id.clone().to_string());


        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Disk {}", 
                requester_api_key.user_id,
                disk_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            CreateDiskResponse::ok(&disk.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn update_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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
        let update_req = serde_json::from_slice::<UpdateDiskRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

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
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
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
            UpdateDiskResponse::ok(&disk.cast_fe(&requester_api_key.user_id)).encode()
        )
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

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        let disk_id = delete_request.id.clone();

        // Check delete permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Disks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.to_string()));
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
                Some(disk.id.to_string()),
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