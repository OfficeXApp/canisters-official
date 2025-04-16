// src/rest/labels/handler.rs

pub mod labels_handlers {
    use crate::{
        core::{
            api::{
                permissions::system::{check_system_permissions, check_system_resource_permissions_labels}, 
                replay::diff::{snapshot_poststate, snapshot_prestate}, 
                uuid::{generate_uuidv4, mark_claimed_uuid}, webhooks::labels::{fire_label_webhook, get_active_label_webhooks}
            },
            state::{
                drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, labels::{
                    state::{
                        add_label_to_resource, parse_label_resource_id, remove_label_from_resource, update_label_string_value, validate_color, validate_label_value, LABELS_BY_ID_HASHTABLE, LABELS_BY_TIME_LIST, LABELS_BY_TIME_MEMORY_ID, LABELS_BY_VALUE_HASHTABLE
                    }, 
                    types::{HexColorString, Label, LabelID, LabelResourceID, LabelStringValue}
                }, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, webhooks::types::WebhookEventLabel
            }, 
            types::IDPrefix
        }, 
        debug_log, 
        rest::{
            auth::{authenticate_request, create_auth_error_response}, 
            labels::types::{
                CreateLabelRequestBody, CreateLabelResponse, DeleteLabelRequest, DeleteLabelResponse, DeletedLabelData, ErrorResponse, GetLabelResponse, LabelOperationResponse, LabelResourceRequest, LabelResourceResponse, ListLabelsRequestBody, ListLabelsResponse, ListLabelsResponseData, UpdateLabelRequestBody, UpdateLabelResponse
            }, 
            webhooks::types::{LabelWebhookData, SortDirection}
        }, MEMORY_MANAGER
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    use ic_stable_structures::StableVec;

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_label_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access private label info
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Get label ID from params
        let label_str = params.get("label_id").unwrap().to_string();
        let label = match label_str.starts_with(&IDPrefix::LabelID.as_str()) {
            // It's a LabelID
            true => {
                let label_id = LabelID(label_str);
                LABELS_BY_ID_HASHTABLE.with(|store| {
                    store.borrow().get(&label_id).clone()
                })
            },
            // It's a LabelStringValue
            false => {
                let label_value = LabelStringValue(label_str);
                // First get the label ID from the value hashtable
                LABELS_BY_VALUE_HASHTABLE.with(|store| {
                    if let Some(label_id) = store.borrow().get(&label_value) {
                        // Then use the label ID to get the full label
                        LABELS_BY_ID_HASHTABLE.with(|id_store| {
                            id_store.borrow().get(&label_id).clone()
                        })
                    } else {
                        None
                    }
                })
            }
        };

        

        match label {
            Some(label) => {
                // Check permissions if not owner
                if !is_owner {
                    // First check table-level permissions
                    let table_resource_id = SystemResourceID::Table(SystemTableEnum::Labels);
                    let table_permissions = check_system_resource_permissions_labels(
                        &table_resource_id,
                        &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                        &label.value.to_string(),
                    );

                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Label(label.id.to_string()));
                     
                    let permissions = check_system_resource_permissions_labels(
                        &resource_id,
                        &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                        &label.value.to_string(),
                    );
                    
                    if !table_permissions.contains(&SystemPermissionType::View) && !permissions.contains(&SystemPermissionType::View) {
                        return create_auth_error_response();
                    }
                }
                create_response(
                    StatusCode::OK,
                    GetLabelResponse::ok(&label.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_labels_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Check if the requester is the owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
    
        // Parse request body
        let body = request.body();
        let request_body: ListLabelsRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }
    
        let prefix_filter = request_body.filters.prefix.as_deref().unwrap_or("");
        
        // Check if user has table-level permission with the given prefix
        let has_table_permission = is_owner || {
            let table_permissions = check_system_resource_permissions_labels(
                &SystemResourceID::Table(SystemTableEnum::Labels),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                prefix_filter
            );
            
            table_permissions.contains(&SystemPermissionType::View)
        };

        debug_log!("has_table_permission: {}", has_table_permission);
    
        // If user doesn't have table-level permissions for this prefix, return early
        if !has_table_permission {
            return create_response(
                StatusCode::OK,
                ListLabelsResponse::ok(&ListLabelsResponseData {
                    items: Vec::new(),
                    page_size: 0,
                    total: 0,
                    direction: request_body.direction,
                    cursor: None,
                }).encode()
            )
        }
    
        // Parse cursor if provided
        let cursor_index = if let Some(cursor) = &request_body.cursor {
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
    
        // First collect all labels that match the filter and permissions
        let mut all_filtered_labels = Vec::new();
        
        LABELS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            LABELS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                for idx in 0..time_index.len() {
                    // Replace time_index[idx] with time_index.get(idx)
                    if let Some(label_id) = time_index.get(idx) {
                        if let Some(label) = id_store.get(&label_id) {
                            // Check resource-level permissions
                            let can_view = is_owner || has_table_permission || {
                                // Check specific permissions for this label
                                let label_id = &label.id;
                                let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Label(label_id.0.clone()));
                                let permissions = check_system_resource_permissions_labels(
                                    &resource_id,
                                    &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                                    &label.value.0
                                );
                                
                                permissions.contains(&SystemPermissionType::View)
                            };
                            
                            if can_view {
                                // Apply prefix filter if provided
                                let meets_prefix_filter = if let Some(prefix) = &request_body.filters.prefix {
                                    label.value.0.to_lowercase().starts_with(&prefix.to_lowercase())
                                } else {
                                    true
                                };
                                
                                if meets_prefix_filter {
                                    all_filtered_labels.push((idx, label.clone()));
                                }
                            }
                        }
                    }
                }
            });
        });
        
        // If there are no matching labels, return early
        if all_filtered_labels.is_empty() {
            return create_response(
                StatusCode::OK,
                ListLabelsResponse::ok(&ListLabelsResponseData {
                    items: vec![],
                    page_size: request_body.page_size,
                    total: 0,
                    direction: request_body.direction,
                    cursor: None,
                }).encode()
            );
        }
        
        // Sort labels based on the requested direction
        match request_body.direction {
            SortDirection::Asc => all_filtered_labels.sort_by(|a, b| a.0.cmp(&b.0)),
            SortDirection::Desc => all_filtered_labels.sort_by(|a, b| b.0.cmp(&a.0)),
        }
        
        let total_filtered_count = all_filtered_labels.len();
        
        // Determine starting point based on cursor
        let start_pos = if let Some(index) = cursor_index {
            match request_body.direction {
                SortDirection::Asc => {
                    // Find position in sorted labels where index >= the cursor value
                    all_filtered_labels.iter().position(|(idx, _)| *idx >= index as u64).unwrap_or(0)
                },
                SortDirection::Desc => {
                    // Find position in sorted labels where index <= the cursor value
                    all_filtered_labels.iter().position(|(idx, _)| *idx <= index as u64).unwrap_or(0)
                },
            }
        } else {
            0 // Start at beginning by default
        };
        
        // Apply pagination
        let page_size = request_body.page_size;
        let end_pos = (start_pos + page_size).min(total_filtered_count);
        
        // Extract the paginated labels
        let paginated_labels: Vec<Label> = all_filtered_labels[start_pos..end_pos]
            .iter()
            .map(|(_, label)| label.clone())
            .collect();
        
        // Calculate next cursor based on direction and where we ended
        let next_cursor = if end_pos < total_filtered_count {
            // There are more items, return cursor for next page
            Some(all_filtered_labels[end_pos].0.to_string())
        } else {
            // No more items
            None
        };
        
        // Calculate total count to return based on permission level
        let total_count_to_return = if is_owner || has_table_permission {
            // Full access users get the actual total count
            total_filtered_count
        } else {
            // Limited access users get the current batch size + 1 if there's more
            if next_cursor.is_some() {
                paginated_labels.len() + 1
            } else {
                paginated_labels.len()
            }
        };
        
        create_response(
            StatusCode::OK,
            ListLabelsResponse::ok(&ListLabelsResponseData {
                items: paginated_labels.into_iter().map(|label| {
                    label.cast_fe(&requester_api_key.user_id)
                }).collect(),
                page_size: page_size,
                total: total_count_to_return,
                direction: request_body.direction,
                cursor: next_cursor,
            }).encode()
        )
    }

    pub async fn create_label_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateLabelRequestBody>(body).unwrap();
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        // Check create permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Labels),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !table_permissions.contains(&SystemPermissionType::Create) {
                return create_auth_error_response();
            }
        }
        
        // Validate label value
        let label_value = match validate_label_value(&create_req.value) {
            Ok(value) => value,
            Err(err) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, err).encode()
            ),
        };
        
        // Check if label already exists
        let label_exists = LABELS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow().contains_key(&label_value)
        });
        
        if label_exists {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Label '{}' already exists", create_req.value)).encode()
            );
        }
        
        // Validate color if provided
        let color = if let Some(color_str) = create_req.color {
            match validate_color(&color_str) {
                Ok(color) => color,
                Err(err) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err).encode()
                ),
            }
        } else {
            HexColorString("#3B82F6".to_string()) // Default blue color
        };
        
        let prestate = snapshot_prestate();

        
        // Create new label

        let label_id = match create_req.id {
            Some(id) => LabelID(id.to_string()),
            None => LabelID(generate_uuidv4(IDPrefix::LabelID)),
        };

        let current_time = ic_cdk::api::time() / 1_000_000;
        let label = Label {
            id: label_id.clone(),
            value: label_value.clone(),
            public_note: create_req.public_note,
            private_note: create_req.private_note,
            color,
            created_by: requester_api_key.user_id.clone(),
            created_at: current_time,
            last_updated_at: current_time,
            resources: vec![],
            labels: vec![],
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };

        // Store the label
        LABELS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(label_id.clone(), label.clone());
        });

        // Store the label value mapping
        LABELS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().insert(label_value, label_id.clone());
        });

        LABELS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(&label_id);
        });
        mark_claimed_uuid(&label_id.clone().to_string());

        update_external_id_mapping(None, label.external_id.clone(), Some(label_id.clone().to_string()));

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Label {}", 
                requester_api_key.user_id,
                label_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            CreateLabelResponse::ok(&label.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn update_label_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateLabelRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        let label_id = LabelID(update_req.id.clone());
                    
        // Get existing label
        let mut label = match LABELS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&label_id).clone()) {
            Some(label) => label,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Check update permission if not owner
        if !is_owner {

            let table_permissions = check_system_resource_permissions_labels(
                &SystemResourceID::Table(SystemTableEnum::Labels),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &label.value.to_string()
            );

            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Label(label_id.to_string()));
            let permissions = check_system_resource_permissions_labels(
                &resource_id,
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &label.value.to_string()
            );
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }
        
        let prestate = snapshot_prestate();

        
        if let Some(public_note) = update_req.public_note {
            label.public_note = Some(public_note);
        }
        
        if let Some(private_note) = update_req.private_note {
            label.private_note = Some(private_note);
        }
        
        if let Some(color_str) = update_req.color {
            match validate_color(&color_str) {
                Ok(color) => {
                    label.color = color;
                },
                Err(err) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err).encode()
                ),
            }
        }
        
        // Update last modified timestamp
        label.last_updated_at = ic_cdk::api::time() / 1_000_000;
        

        // Update fields
        if let Some(value_str) = update_req.value {
            match validate_label_value(&value_str) {
                Ok(new_value) => {
                    
                    // Update all resources using the label using our helper function
                    if let Err(err) = update_label_string_value(&label_id,  &new_value) {
                        return create_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ErrorResponse::err(500, err).encode()
                        );
                    }
                    
                    // Update the label with new value
                    label.value = new_value.clone();
                },
                Err(err) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err).encode()
                ),
            }
        }

        if let Some(external_id) = update_req.external_id.clone() {
            let old_external_id = label.external_id.clone();
            let new_external_id = Some(ExternalID(external_id.clone()));
            label.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(label.id.to_string())
            );
        }
        if let Some(external_payload) = update_req.external_payload.clone() {
            label.external_payload = Some(ExternalPayload(external_payload));
        }

        LABELS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(label_id.clone(), label.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Label {}", 
                requester_api_key.user_id,
                label_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            UpdateLabelResponse::ok(&label.cast_fe(&requester_api_key.user_id)).encode()
        )
    }


    pub async fn delete_label_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteLabelRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        let label_id = LabelID(delete_request.id.clone());

        // Check if label exists
        let label = LABELS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&label_id).clone()
        });
        
        let label = match label {
            Some(label) => label,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = label.external_id.clone();
        let old_internal_id = Some(label_id.clone().to_string());

        // Check delete permission if not owner
        if !is_owner {

            let table_permissions = check_system_resource_permissions_labels(
                &SystemResourceID::Table(SystemTableEnum::Labels),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &label.value.to_string()
            );

            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Label(label_id.to_string()));
            let permissions = check_system_resource_permissions_labels(
                &resource_id,
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &label.value.to_string()
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // Remove from value mapping
        LABELS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&label.value);
        });

        // Remove from main stores
        LABELS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&label_id);
        });

        LABELS_BY_TIME_LIST.with(|store| {
            let mut new_vec = StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(LABELS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize new StableVec");
            
            // Copy all items except the one to be deleted
            let store_ref = store.borrow();
            for i in 0..store_ref.len() {
                if let Some(id) = store_ref.get(i) {
                    if id != label_id {
                        new_vec.push(&id);
                    }
                }
            }
            
            // Replace the old vector with the new one
            drop(store_ref);
            *store.borrow_mut() = new_vec;
        });

        update_external_id_mapping(old_external_id, None, old_internal_id);

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Label {}", 
                requester_api_key.user_id,
                label_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            DeleteLabelResponse::ok(&DeletedLabelData {
                id: label_id,
                deleted: true
            }).encode()
        )
    }

    pub async fn label_pin_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Parse request body
        let body: &[u8] = request.body();
        let label_request = match serde_json::from_slice::<LabelResourceRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = label_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        // Parse the label ID
        let label_id = match LABELS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&LabelID(label_request.label_id.clone())).clone()
        }) {
            Some(label) => label.id,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, format!("Label with ID {} not found", label_request.label_id)).encode()
            ),
        };
        
        // Parse the resource ID
        let resource_id = match parse_label_resource_id(&label_request.resource_id) {
            Ok(resource_id) => resource_id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Invalid resource ID: {}", label_request.resource_id)).encode()
            ),
        };

        
        let prestate = snapshot_prestate();

        // Get the label value
        // let label = LABELS_BY_ID_HASHTABLE.with(|store| {
        //     store.borrow().get(&label_id).map(|label| label.clone())
        // }).unwrap();

        // check if label exists, throw bad request if not
        let label = match LABELS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&label_id).clone()) {
            Some(label) => label,
            None => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Label with ID {} not found", label_id.0)).encode()
            ),
        };
        let label_value = label.value.clone();


        // Check update permission on the resource
        if !is_owner {
            let table_permissions = check_system_resource_permissions_labels(
                &SystemResourceID::Table(SystemTableEnum::Labels),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &label_value.to_string()
            );

            let system_resource_id = SystemResourceID::Record(SystemRecordIDEnum::Label(resource_id.get_id_string()));
            let permissions = check_system_resource_permissions_labels(
                &system_resource_id,
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &label_value.to_string()
            );
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }

        let result = if label_request.add {
            // Add label to resource
            add_label_to_resource(&resource_id, &label_value)
        } else {
            // Remove label from resource
            remove_label_from_resource(&resource_id, &label_value)
        };
        
        match result {
            Ok(_) => {

                let after_snap = LabelWebhookData {
                    label_id: label_id.clone(),
                    resource_id: resource_id.clone(),
                    label_value: label_value.clone(),
                    add: label_request.add,
                };
                
                // Determine webhook event type based on action
                let webhook_event = if label_request.add {
                    WebhookEventLabel::LabelAdded
                } else {
                    WebhookEventLabel::LabelRemoved
                };
                
                // Get active webhooks for this label
                let webhooks = get_active_label_webhooks(&label_id, webhook_event.clone());
                
                // Fire webhook if there are active webhooks
                if !webhooks.is_empty() {
                    let notes = Some(format!(
                        "Label {} {} resource {}", 
                        if label_request.add { "added to" } else { "removed from" },
                        label_id.0.clone(),
                        resource_id.get_id_string()
                    ));
                    
                    fire_label_webhook(
                        webhook_event,
                        webhooks,
                        None,
                        Some(after_snap),
                        notes
                    );
                }
                
                
                let action = if label_request.add { "Add" } else { "Remove" };
                snapshot_poststate(prestate, Some(
                    format!(
                        "{}: {} Label {} to Resource {}", 
                        requester_api_key.user_id,
                        action,
                        label_id.clone(),
                        resource_id.get_id_string()
                    ).to_string())
                );

                
                
                create_response(
                    StatusCode::OK,
                    LabelResourceResponse::ok(&LabelOperationResponse {
                        success: true,
                        message: Some(format!("Successfully {}ed label", if label_request.add { "add" } else { "remov" })),
                        label: Some(label.cast_fe(&requester_api_key.user_id)),
                    }).encode()
                )
            },
            Err(err) => create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, err).encode()
            ),
        }
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

    fn json_decode<T>(value: &[u8]) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_slice(value).expect("Failed to deserialize value")
    }
}