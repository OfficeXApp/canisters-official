// src/rest/tags/handler.rs

pub mod tags_handlers {
    use crate::{
        core::{
            api::{
                permissions::system::{check_system_permissions, check_system_resource_permissions_tags}, 
                replay::diff::{snapshot_poststate, snapshot_prestate}, 
                uuid::generate_unique_id, webhooks::tags::{fire_tag_webhook, get_active_tag_webhooks}
            },
            state::{
                drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, 
                permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, 
                tags::{
                    state::{
                        add_tag_to_resource, 
                        parse_tag_resource_id, 
                        remove_tag_from_resource, 
                        update_tag_string_value, 
                        validate_color, 
                        validate_tag_value, 
                        TAGS_BY_ID_HASHTABLE, 
                        TAGS_BY_TIME_LIST, 
                        TAGS_BY_VALUE_HASHTABLE
                    }, 
                    types::{HexColorString, Tag, TagID, TagResourceID, TagStringValue}
                }, webhooks::types::WebhookEventLabel
            }, 
            types::IDPrefix
        }, 
        debug_log, 
        rest::{
            auth::{authenticate_request, create_auth_error_response}, 
            tags::types::{
                CreateTagRequestBody, CreateTagResponse, DeleteTagRequest, DeleteTagResponse, DeletedTagData, ErrorResponse, GetTagResponse, ListTagsRequestBody, ListTagsResponse, ListTagsResponseData, TagOperationResponse, TagResourceRequest, TagResourceResponse, UpdateTagRequestBody, UpdateTagResponse
            }, 
            webhooks::types::{SortDirection, TagWebhookData}
        }
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_tag_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access private tag info
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Get tag ID from params
        let tag_str = params.get("tag_id").unwrap().to_string();
        let tag = match tag_str.starts_with(&IDPrefix::TagID.as_str()) {
            // It's a TagID
            true => {
                let tag_id = TagID(tag_str);
                TAGS_BY_ID_HASHTABLE.with(|store| {
                    store.borrow().get(&tag_id).cloned()
                })
            },
            // It's a TagStringValue
            false => {
                let tag_value = TagStringValue(tag_str);
                // First get the tag ID from the value hashtable
                TAGS_BY_VALUE_HASHTABLE.with(|store| {
                    if let Some(tag_id) = store.borrow().get(&tag_value) {
                        // Then use the tag ID to get the full tag
                        TAGS_BY_ID_HASHTABLE.with(|id_store| {
                            id_store.borrow().get(tag_id).cloned()
                        })
                    } else {
                        None
                    }
                })
            }
        };

        

        match tag {
            Some(tag) => {
                // Check permissions if not owner
                if !is_owner {
                    // First check table-level permissions
                    let table_resource_id = SystemResourceID::Table(SystemTableEnum::Tags);
                    let table_permissions = check_system_resource_permissions_tags(
                        &table_resource_id,
                        &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                        &tag.value.to_string(),
                    );

                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Tag(tag.id.to_string()));
                     
                    let permissions = check_system_resource_permissions_tags(
                        &resource_id,
                        &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                        &tag.value.to_string(),
                    );
                    
                    if !table_permissions.contains(&SystemPermissionType::View) && !permissions.contains(&SystemPermissionType::View) {
                        return create_auth_error_response();
                    }
                }
                create_response(
                    StatusCode::OK,
                    GetTagResponse::ok(&tag.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_tags_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
    
        // Parse request body
        let body = request.body();
        let request_body: ListTagsRequestBody = match serde_json::from_slice(body) {
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
        
        // If not owner, check early if user has permission to search with the given prefix
        if !is_owner {
            let table_permissions = check_system_resource_permissions_tags(
                &SystemResourceID::Table(SystemTableEnum::Tags),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                prefix_filter
            );
            
            // Throw early error if user doesn't have permission to search with this prefix
            if !table_permissions.contains(&SystemPermissionType::View) {
                return create_response(
                    StatusCode::FORBIDDEN,
                    ErrorResponse::err(403, format!("You don't have permission to search tags with prefix '{}'", prefix_filter)).encode()
                );
            }
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
    
        // First collect all tags that match the filter
        let mut all_filtered_tags = Vec::new();
        
        TAGS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            TAGS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                for idx in 0..time_index.len() {
                    if let Some(tag) = id_store.get(&time_index[idx]) {
                        // Check record-level permissions for non-owners
                        let has_access = is_owner || {
                            // For non-owners, check record-level permissions
                            let tag_id = &tag.id;
                            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Tag(tag_id.0.clone()));
                            let permissions = check_system_resource_permissions_tags(
                                &resource_id,
                                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                                &tag.value.0
                            );
                            // We already checked table-level permissions earlier,
                            // so we only need to check if there are any record-specific
                            // permissions that explicitly deny access
                            permissions.is_empty() || permissions.contains(&SystemPermissionType::View)
                        };
                        
                        if has_access {
                            // Apply prefix filter if provided
                            let meets_prefix_filter = if let Some(prefix) = &request_body.filters.prefix {
                                tag.value.0.to_lowercase().starts_with(&prefix.to_lowercase())
                            } else {
                                true
                            };
                            
                            if meets_prefix_filter {
                                all_filtered_tags.push((idx, tag.clone()));
                            }
                        }
                    }
                }
            });
        });
        
        // If there are no matching tags, return early
        if all_filtered_tags.is_empty() {
            return create_response(
                StatusCode::OK,
                ListTagsResponse::ok(&ListTagsResponseData {
                    items: vec![],
                    page_size: request_body.page_size,
                    total: 0,
                    cursor_up: None,
                    cursor_down: None,
                }).encode()
            );
        }
        
        // Sort tags based on the requested direction
        match request_body.direction {
            SortDirection::Asc => all_filtered_tags.sort_by(|a, b| a.0.cmp(&b.0)),
            SortDirection::Desc => all_filtered_tags.sort_by(|a, b| b.0.cmp(&a.0)),
        }
        
        let total_filtered_count = all_filtered_tags.len();
        
        // Determine starting point based on cursors
        let start_pos = if let Some(up) = cursor_up {
            // Find position in filtered tags where index >= up
            match request_body.direction {
                SortDirection::Asc => all_filtered_tags.iter().position(|(idx, _)| *idx >= up).unwrap_or(0),
                SortDirection::Desc => all_filtered_tags.iter().position(|(idx, _)| *idx <= up).unwrap_or(0),
            }
        } else if let Some(down) = cursor_down {
            // Find position in filtered tags where index <= down
            match request_body.direction {
                SortDirection::Asc => all_filtered_tags.iter().position(|(idx, _)| *idx <= down)
                    .map(|pos| if pos > 0 { pos - 1 } else { 0 })
                    .unwrap_or(0),
                SortDirection::Desc => all_filtered_tags.iter().position(|(idx, _)| *idx >= down)
                    .map(|pos| if pos > 0 { pos - 1 } else { 0 })
                    .unwrap_or(0),
            }
        } else {
            0 // Start at beginning by default
        };
        
        // Apply pagination
        let page_size = request_body.page_size;
        let end_pos = (start_pos + page_size).min(total_filtered_count);
        
        // Extract the paginated tags
        let paginated_tags: Vec<Tag> = all_filtered_tags[start_pos..end_pos]
            .iter()
            .map(|(_, tag)| tag.clone())
            .collect();
        
        // Calculate next cursors
        let cursor_up = if end_pos < total_filtered_count {
            Some(all_filtered_tags[end_pos].0.to_string())
        } else {
            None
        };
        
        let cursor_down = if start_pos > 0 {
            Some(all_filtered_tags[start_pos - 1].0.to_string())
        } else {
            None
        };
        
        create_response(
            StatusCode::OK,
            ListTagsResponse::ok(&ListTagsResponseData {
                items: paginated_tags.into_iter().map(|tag| {
                    tag.cast_fe(&requester_api_key.user_id)
                }).collect(),
                page_size: page_size,
                total: total_filtered_count,
                cursor_up,
                cursor_down,
            }).encode()
        )
    }

    pub async fn create_tag_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateTagRequestBody>(body).unwrap();
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        // Check create permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Tags),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !table_permissions.contains(&SystemPermissionType::Create) {
                return create_auth_error_response();
            }
        }
        
        // Validate tag value
        let tag_value = match validate_tag_value(&create_req.value) {
            Ok(value) => value,
            Err(err) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, err).encode()
            ),
        };
        
        // Check if tag already exists
        let tag_exists = TAGS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow().contains_key(&tag_value)
        });
        
        if tag_exists {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Tag '{}' already exists", create_req.value)).encode()
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

        
        // Create new tag
        let tag_id = TagID(generate_unique_id(IDPrefix::TagID, ""));
        let current_time = ic_cdk::api::time() / 1_000_000;
        let tag = Tag {
            id: tag_id.clone(),
            value: tag_value.clone(),
            public_note: create_req.public_note,
            private_note: create_req.private_note,
            color,
            created_by: requester_api_key.user_id.clone(),
            created_at: current_time,
            last_updated_at: current_time,
            resources: vec![],
            tags: vec![],
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };

        // Store the tag
        TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(tag_id.clone(), tag.clone());
        });

        // Store the tag value mapping
        TAGS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().insert(tag_value, tag_id.clone());
        });

        TAGS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(tag_id.clone());
        });

        update_external_id_mapping(None, tag.external_id.clone(), Some(tag_id.clone().to_string()));

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Tag {}", 
                requester_api_key.user_id,
                tag_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            CreateTagResponse::ok(&tag.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn update_tag_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateTagRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        let tag_id = TagID(update_req.id.clone());
                    
        // Get existing tag
        let mut tag = match TAGS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&tag_id).cloned()) {
            Some(tag) => tag,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Check update permission if not owner
        if !is_owner {

            let table_permissions = check_system_resource_permissions_tags(
                &SystemResourceID::Table(SystemTableEnum::Tags),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &tag.value.to_string()
            );

            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Tag(tag_id.to_string()));
            let permissions = check_system_resource_permissions_tags(
                &resource_id,
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &tag.value.to_string()
            );
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }
        
        let prestate = snapshot_prestate();

        
        if let Some(public_note) = update_req.public_note {
            tag.public_note = Some(public_note);
        }
        
        if let Some(private_note) = update_req.private_note {
            tag.private_note = Some(private_note);
        }
        
        if let Some(color_str) = update_req.color {
            match validate_color(&color_str) {
                Ok(color) => {
                    tag.color = color;
                },
                Err(err) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err).encode()
                ),
            }
        }
        
        // Update last modified timestamp
        tag.last_updated_at = ic_cdk::api::time() / 1_000_000;
        

        // Update fields
        if let Some(value_str) = update_req.value {
            match validate_tag_value(&value_str) {
                Ok(new_value) => {
                    
                    // Update all resources using the tag using our helper function
                    if let Err(err) = update_tag_string_value(&tag_id,  &new_value) {
                        return create_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ErrorResponse::err(500, err).encode()
                        );
                    }
                    
                    // Update the tag with new value
                    tag.value = new_value.clone();
                },
                Err(err) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err).encode()
                ),
            }
        }

        if let Some(external_id) = update_req.external_id.clone() {
            let old_external_id = tag.external_id.clone();
            let new_external_id = Some(ExternalID(external_id.clone()));
            tag.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(tag.id.to_string())
            );
        }
        if let Some(external_payload) = update_req.external_payload.clone() {
            tag.external_payload = Some(ExternalPayload(external_payload));
        }

        TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(tag_id.clone(), tag.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Tag {}", 
                requester_api_key.user_id,
                tag_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            UpdateTagResponse::ok(&tag.cast_fe(&requester_api_key.user_id)).encode()
        )
    }


    pub async fn delete_tag_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteTagRequest>(body) {
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

        let tag_id = TagID(delete_request.id.clone());

        // Check if tag exists
        let tag = TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&tag_id).cloned()
        });
        
        let tag = match tag {
            Some(tag) => tag,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = tag.external_id.clone();
        let old_internal_id = Some(tag_id.clone().to_string());

        // Check delete permission if not owner
        if !is_owner {

            let table_permissions = check_system_resource_permissions_tags(
                &SystemResourceID::Table(SystemTableEnum::Tags),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &tag.value.to_string()
            );

            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Tag(tag_id.to_string()));
            let permissions = check_system_resource_permissions_tags(
                &resource_id,
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &tag.value.to_string()
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // Remove from value mapping
        TAGS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&tag.value);
        });

        // Remove from main stores
        TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&tag_id);
        });

        TAGS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &tag_id);
        });

        update_external_id_mapping(old_external_id, None, old_internal_id);

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Tag {}", 
                requester_api_key.user_id,
                tag_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            DeleteTagResponse::ok(&DeletedTagData {
                id: tag_id,
                deleted: true
            }).encode()
        )
    }

    pub async fn tag_pin_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let tag_request = match serde_json::from_slice::<TagResourceRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = tag_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("{}: {}", validation_error.field, validation_error.message)).encode()
            );
        }

        // Parse the tag ID
        let tag_id = match TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&TagID(tag_request.tag_id.clone())).cloned()
        }) {
            Some(tag) => tag.id,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, format!("Tag with ID {} not found", tag_request.tag_id)).encode()
            ),
        };
        
        // Parse the resource ID
        let resource_id = match parse_tag_resource_id(&tag_request.resource_id) {
            Ok(resource_id) => resource_id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Invalid resource ID: {}", tag_request.resource_id)).encode()
            ),
        };

        
        let prestate = snapshot_prestate();

        // Get the tag value
        // let tag = TAGS_BY_ID_HASHTABLE.with(|store| {
        //     store.borrow().get(&tag_id).map(|tag| tag.clone())
        // }).unwrap();

        // check if tag exists, throw bad request if not
        let tag = match TAGS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&tag_id).cloned()) {
            Some(tag) => tag,
            None => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Tag with ID {} not found", tag_id.0)).encode()
            ),
        };
        let tag_value = tag.value.clone();


        // Check update permission on the resource
        if !is_owner {
            let table_permissions = check_system_resource_permissions_tags(
                &SystemResourceID::Table(SystemTableEnum::Tags),
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &tag_value.to_string()
            );

            let system_resource_id = SystemResourceID::Record(SystemRecordIDEnum::Tag(resource_id.get_id_string()));
            let permissions = check_system_resource_permissions_tags(
                &system_resource_id,
                &PermissionGranteeID::User(requester_api_key.user_id.clone()),
                &tag_value.to_string()
            );
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }

        let result = if tag_request.add {
            // Add tag to resource
            add_tag_to_resource(&resource_id, &tag_value)
        } else {
            // Remove tag from resource
            remove_tag_from_resource(&resource_id, &tag_value)
        };
        
        match result {
            Ok(_) => {

                let after_snap = TagWebhookData {
                    tag_id: tag_id.clone(),
                    resource_id: resource_id.clone(),
                    tag_value: tag_value.clone(),
                    add: tag_request.add,
                };
                
                // Determine webhook event type based on action
                let webhook_event = if tag_request.add {
                    WebhookEventLabel::TagAdded
                } else {
                    WebhookEventLabel::TagRemoved
                };
                
                // Get active webhooks for this tag
                let webhooks = get_active_tag_webhooks(&tag_id, webhook_event.clone());
                
                // Fire webhook if there are active webhooks
                if !webhooks.is_empty() {
                    let notes = Some(format!(
                        "Tag {} {} resource {}", 
                        if tag_request.add { "added to" } else { "removed from" },
                        tag_id.0.clone(),
                        resource_id.get_id_string()
                    ));
                    
                    fire_tag_webhook(
                        webhook_event,
                        webhooks,
                        None,
                        Some(after_snap),
                        notes
                    );
                }
                
                
                let action = if tag_request.add { "Add" } else { "Remove" };
                snapshot_poststate(prestate, Some(
                    format!(
                        "{}: {} Tag {} to Resource {}", 
                        requester_api_key.user_id,
                        action,
                        tag_id.clone(),
                        resource_id.get_id_string()
                    ).to_string())
                );

                
                
                create_response(
                    StatusCode::OK,
                    TagResourceResponse::ok(&TagOperationResponse {
                        success: true,
                        message: Some(format!("Successfully {}ed tag", if tag_request.add { "add" } else { "remov" })),
                        tag: Some(tag.cast_fe(&requester_api_key.user_id)),
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