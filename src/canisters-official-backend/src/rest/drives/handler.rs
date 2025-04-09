// src/rest/drives/handler.rs


pub mod drives_handlers {
    use crate::{
        core::{api::{permissions::{directory::{can_user_access_directory_permission, check_directory_permissions}, system::{can_user_access_system_permission, check_system_permissions}}, replay::diff::{apply_state_diff, safely_apply_diffs, snapshot_entire_state, snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}}, state::{api_keys::state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, disks::state::state::{DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, drives::{state::state::{update_external_id_mapping, DRIVES_BY_ID_HASHTABLE, DRIVES_BY_TIME_LIST, DRIVE_ID, DRIVE_STATE_CHECKSUM, DRIVE_STATE_TIMESTAMP_NS, EXTERNAL_ID_MAPPINGS, OWNER_ID, TRANSFER_OWNER_ID, URL_ENDPOINT}, types::{Drive, DriveID, DriveRESTUrlEndpoint, DriveStateDiffID, ExternalID, ExternalPayload}}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE}, types::{DirectoryPermissionType, PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, search::types::SearchCategoryEnum, labels::{state::{add_label_to_resource, parse_label_resource_id, remove_label_from_resource, validate_label_value}, types::{LabelOperationResponse, LabelResourceID}}, group_invites::state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, groups::state::state::{is_group_admin, GROUPS_BY_ID_HASHTABLE, GROUPS_BY_TIME_LIST}}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::DirectoryResourceID, drives::types::{CreateDriveRequestBody, CreateDriveResponse, DeleteDriveRequest, DeleteDriveResponse, DeletedDriveData, ErrorResponse, GetDriveResponse, ListDrivesRequestBody, ListDrivesResponse, ListDrivesResponseData, UpdateDriveRequestBody, UpdateDriveResponse}, webhooks::types::SortDirection}
        
    };
    use ic_types::crypto::canister_threshold_sig::PublicKey;
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
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(drive_id.to_string()));
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
                    GetDriveResponse::ok(&drive.cast_fe(&requester_api_key.user_id)).encode()
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
    
        // Check if the requester is the owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
    
        // Check table-level permissions if not owner
        let has_table_permission = if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Drives);
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
        let request_body: ListDrivesRequestBody = match serde_json::from_slice(body) {
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
        let total_count = DRIVES_BY_TIME_LIST.with(|list| list.borrow().len());
    
        // If there are no drives, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListDrivesResponse::ok(&ListDrivesResponseData {
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
    
        // Get drives with pagination and filtering, applying permission checks
        let mut filtered_drives = Vec::new();
        let mut processed_count = 0;
        let mut end_index = start_index;  // Track where we ended for cursor calculation
        let mut total_count_to_return = 0; // Will use this for the response
    
        // If user is owner or has table access, they get the actual total count
        if is_owner || has_table_permission {
            total_count_to_return = total_count;
        }
    
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
                                // Check if user has permission to view this specific drive
                                let can_view = is_owner || has_table_permission || {
                                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Drive(drive.id.to_string()));
                                    let permissions = check_system_permissions(
                                        resource_id,
                                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                                    );
                                    permissions.contains(&SystemPermissionType::View)
                                };
    
                                if can_view && request_body.filters.is_empty() {
                                    filtered_drives.push(drive.clone());
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
                        // Oldest first
                        let mut current_idx = start_index;
                        while filtered_drives.len() < request_body.page_size && current_idx < total_count {
                            if let Some(drive) = id_store.get(&time_index[current_idx]) {
                                // Check if user has permission to view this specific drive
                                let can_view = is_owner || has_table_permission || {
                                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Drive(drive.id.to_string()));
                                    let permissions = check_system_permissions(
                                        resource_id,
                                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                                    );
                                    permissions.contains(&SystemPermissionType::View)
                                };
    
                                if can_view && request_body.filters.is_empty() {
                                    filtered_drives.push(drive.clone());
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
        let next_cursor = if filtered_drives.len() >= request_body.page_size {
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
                total_count_to_return = filtered_drives.len() + 1;
            } else {
                // Otherwise, just return the batch size
                total_count_to_return = filtered_drives.len();
            }
        }
    
        // Create response
        let response_data = ListDrivesResponseData {
            items: filtered_drives.clone().into_iter().map(|drive| {
                drive.cast_fe(&requester_api_key.user_id)
            }).collect(),
            page_size: filtered_drives.len(),
            total: total_count_to_return,
            direction: request_body.direction,
            cursor: next_cursor,
        };
    
        create_response(
            StatusCode::OK,
            ListDrivesResponse::ok(&response_data).encode()
        )
    }

    pub async fn create_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
       

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateDriveRequestBody>(body).unwrap();
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }

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

        let drive_id = match create_req.id {
            Some(id) => DriveID(id.to_string()),
            None => DriveID(generate_uuidv4(IDPrefix::Drive)),
        };

        let drive = Drive {
            id: drive_id.clone(),
            name: create_req.name,
            public_note: Some(create_req.public_note.unwrap_or_default()),
            private_note: Some(create_req.private_note.unwrap_or_default()),
            icp_principal: ICPPrincipalString(PublicKeyICP(create_req.icp_principal)),
            endpoint_url: DriveRESTUrlEndpoint(
                create_req.endpoint_url
                    .unwrap_or(URL_ENDPOINT.with(|url| url.borrow().clone()).0)
                    .trim_end_matches('/')
                    .to_string()
            ),
            last_indexed_ms: None,
            labels: vec![],
            created_at: ic_cdk::api::time() / 1_000_000,
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
        mark_claimed_uuid(&drive_id.clone().to_string());

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Drive {}", 
                requester_api_key.user_id,
                drive_id.clone()
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            CreateDriveResponse::ok(&drive.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn update_drive_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
       

        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateDriveRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }

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
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(drive_id.to_string()));
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
        if let Some(name) = update_req.name {
            drive.name = name;
        }
        if let Some(public_note) = update_req.public_note {
            drive.public_note = Some(public_note);
        }
        if let Some(private_note) = update_req.private_note {
            drive.private_note = Some(private_note);
        }
        if let Some(endpoint_url) = update_req.endpoint_url {
            drive.endpoint_url = DriveRESTUrlEndpoint(endpoint_url.trim_end_matches('/')
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
            UpdateDriveResponse::ok(&drive.cast_fe(&requester_api_key.user_id)).encode()
        )
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

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Validation error: {}: {}", 
                    validation_error.field, validation_error.message)).encode()
            );
        }

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
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(drive_id.to_string()));
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