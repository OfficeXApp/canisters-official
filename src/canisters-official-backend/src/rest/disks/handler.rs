// src/rest/disks/handler.rs


pub mod disks_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{disks::{state::state::{DISKS_BY_EXTERNAL_ID_HASHTABLE, DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, types::{Disk, DiskID}}, drives::state::state::OWNER_ID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, disks::types::{ CreateDiskResponse, DeleteDiskRequest, DeleteDiskResponse, DeletedDiskData, ErrorResponse, GetDiskResponse, ListDisksRequestBody, ListDisksResponse, ListDisksResponseData, UpdateDiskResponse, UpsertDiskRequestBody}, webhooks::types::SortDirection}
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_disk_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access disk.private_note
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
        }

        // Get disk ID from params
        let disk_id = DiskID(params.get("disk_id").unwrap().to_string());

        // Get the disk
        let disk = DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&disk_id).cloned()
        });

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

    pub fn list_disks_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
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


    pub fn upsert_disk_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body: &[u8] = req.body();

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

                    // Update fields
                    if let Some(name) = update_req.name {
                        disk.name = name;
                    }
                    if let Some(public_note) = update_req.public_note {
                        disk.public_note = Some(public_note);
                    }
                    if let Some(private_note) = update_req.private_note {
                        disk.private_note = Some(private_note);
                    }
                    if let Some(auth_json) = update_req.auth_json {
                        disk.auth_json = Some(auth_json);
                    }
                    if let Some(external_id) = update_req.external_id {
                        // Update external ID mapping
                        if let Some(old_external_id) = &disk.external_id {
                            DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|store| {
                                store.borrow_mut().remove(old_external_id);
                            });
                        }
                        DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|store| {
                            store.borrow_mut().insert(external_id.clone(), disk_id.clone());
                        });
                        disk.external_id = Some(external_id);
                    }

                    DISKS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(disk_id.clone(), disk.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateDiskResponse::ok(&disk).encode()
                    )
                },
                UpsertDiskRequestBody::Create(create_req) => {
                    // Create new disk
                    let disk_type_suffix = format!("--DiskType_{}", create_req.disk_type);
                    let disk_id = DiskID(generate_unique_id("DiskID", &disk_type_suffix));
                    let disk = Disk {
                        id: disk_id.clone(),
                        name: create_req.name,
                        public_note: create_req.public_note,
                        private_note: create_req.private_note,
                        auth_json: create_req.auth_json,
                        disk_type: create_req.disk_type,
                        external_id: create_req.external_id.clone(),
                    };

                    // Store the disk
                    DISKS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(disk_id.clone(), disk.clone());
                    });

                    // Store external ID mapping if provided
                    if let Some(external_id) = &disk.external_id {
                        DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|store| {
                            store.borrow_mut().insert(external_id.clone(), disk_id.clone());
                        });
                    }

                    DISKS_BY_TIME_LIST.with(|store| {
                        store.borrow_mut().push(disk_id.clone());
                    });

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

    pub fn delete_disk_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
        }

        // Parse request body
        let body: &[u8] = req.body();
        let delete_request = match serde_json::from_slice::<DeleteDiskRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        let disk_id = delete_request.id.clone();

        // Get disk for external ID cleanup
        let disk = DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&disk_id).cloned()
        });

        // Remove from external ID mapping if it exists
        if let Some(disk) = disk {
            if let Some(external_id) = disk.external_id {
                DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|store| {
                    store.borrow_mut().remove(&external_id);
                });
            }
        }

        // Remove from main stores
        DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&disk_id);
        });

        DISKS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &disk_id);
        });

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