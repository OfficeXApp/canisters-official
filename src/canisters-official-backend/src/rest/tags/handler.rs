// src/rest/tags/handler.rs

pub mod tags_handlers {
    use crate::{
        core::{
            api::{
                permissions::system::check_system_permissions, 
                replay::diff::{snapshot_poststate, snapshot_prestate}, 
                uuid::generate_unique_id
            },
            state::{
                drives::state::state::OWNER_ID, 
                permissions::types::{PermissionGranteeID, SystemPermissionType, SystemResourceID, SystemTableEnum}, 
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
                    types::{HexColorString, Tag, TagID, TagStringValue, TagResourceID}
                }
            }, 
            types::IDPrefix
        }, 
        debug_log, 
        rest::{
            auth::{authenticate_request, create_auth_error_response}, 
            tags::types::{
                CreateTagResponse, 
                DeleteTagResponse, 
                DeletedTagData, 
                ErrorResponse, 
                GetTagResponse, 
                ListTagsResponse, 
                ListTagsResponseData,
                UpdateTagResponse,
                ListTagsRequestBody,
                UpsertTagRequestBody,
                CreateTagRequestBody,
                UpdateTagRequestBody,
                DeleteTagRequest,
                TagResourceRequest,
                TagOperationResponse,
                TagResourceResponse
            }, 
            webhooks::types::SortDirection
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
        let tag_id = TagID(params.get("tag_id").unwrap().to_string());

        // Get the tag
        let tag = TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&tag_id).cloned()
        });

        // Check permissions if not owner
        if !is_owner {
            // First check table-level permissions
            let table_resource_id = SystemResourceID::Table(SystemTableEnum::Tags);
            let table_permissions = check_system_permissions(
                table_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            let resource_id = SystemResourceID::Record(tag_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !table_permissions.contains(&SystemPermissionType::View) && !permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        match tag {
            Some(tag) => {
                create_response(
                    StatusCode::OK,
                    GetTagResponse::ok(&tag).encode()
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

        // Check table permissions if not owner
        if !is_owner {
            
            let resource_id = SystemResourceID::Table(SystemTableEnum::Tags);
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
        let request_body: ListTagsRequestBody = match serde_json::from_slice(body) {
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
        let total_count = TAGS_BY_TIME_LIST.with(|list| list.borrow().len());

        // If there are no tags, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListTagsResponse::ok(&ListTagsResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    cursor_up: None,
                    cursor_down: None,
                }).encode()
            );
        }

        let page_size = request_body.page_size;

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

        // Get tags with pagination and filtering
        let mut filtered_tags = Vec::new();
        let mut processed_count = 0;

        TAGS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            TAGS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_tags.len() < page_size && current_idx < total_count {
                            if let Some(tag) = id_store.get(&time_index[current_idx]) {
                                // Apply filter if provided
                                let should_include = if !request_body.filters.is_empty() {
                                    tag.value.0.to_lowercase().contains(&request_body.filters.to_lowercase()) ||
                                    tag.description.as_ref().map(|d| d.to_lowercase().contains(&request_body.filters.to_lowercase())).unwrap_or(false)
                                } else {
                                    true
                                };
                                
                                if should_include {
                                    filtered_tags.push(tag.clone());
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
                        while filtered_tags.len() < page_size && current_idx < total_count {
                            if let Some(tag) = id_store.get(&time_index[current_idx]) {
                                // Apply filter if provided
                                let should_include = if !request_body.filters.is_empty() {
                                    tag.value.0.to_lowercase().contains(&request_body.filters.to_lowercase()) ||
                                    tag.description.as_ref().map(|d| d.to_lowercase().contains(&request_body.filters.to_lowercase())).unwrap_or(false)
                                } else {
                                    true
                                };
                                
                                if should_include {
                                    filtered_tags.push(tag.clone());
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
            ListTagsResponse::ok(&ListTagsResponseData {
                items: filtered_tags,
                page_size: page_size,
                total: total_count,
                cursor_up,
                cursor_down,
            }).encode()
        )
    }

    pub async fn upsert_tag_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();

        if let Ok(req) = serde_json::from_slice::<UpsertTagRequestBody>(body) {
            match req {
                UpsertTagRequestBody::Update(update_req) => {
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

                        let table_permissions = check_system_permissions(
                            SystemResourceID::Table(SystemTableEnum::Tags),
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );

                        let resource_id = SystemResourceID::Record(tag_id.to_string());
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Update) && !table_permissions.contains(&SystemPermissionType::Update) {
                            return create_auth_error_response();
                        }
                    }
                    
                    let prestate = snapshot_prestate();

                    
                    if let Some(description) = update_req.description {
                        tag.description = Some(description);
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
                        UpdateTagResponse::ok(&tag).encode()
                    )
                },
                UpsertTagRequestBody::Create(create_req) => {
                    // Check create permission if not owner
                    if !is_owner {
                        let table_permissions = check_system_permissions(
                            SystemResourceID::Table(SystemTableEnum::Tags),
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );

                        let resource_id = SystemResourceID::Table(SystemTableEnum::Tags);
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Create) && !table_permissions.contains(&SystemPermissionType::Create) {
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
                        description: create_req.description,
                        color,
                        created_by: requester_api_key.user_id.clone(),
                        created_at: current_time,
                        last_updated_at: current_time,
                        resources: vec![],
                        tags: vec![],
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

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Create Tag {}", 
                            requester_api_key.user_id,
                            tag_id.clone()
                        ).to_string())
                    );

                    create_response(
                        StatusCode::OK,
                        CreateTagResponse::ok(&tag).encode()
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

        // Check delete permission if not owner
        if !is_owner {

            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Tags),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            let resource_id = SystemResourceID::Record(tag_id.to_string());
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
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

    pub async fn tag_resource_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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

        // Check update permission on the resource
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Tags),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            let system_resource_id = SystemResourceID::Record(resource_id.get_id_string());
            let permissions = check_system_permissions(
                system_resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Update) && !table_permissions.contains(&SystemPermissionType::Update) {
                return create_auth_error_response();
            }
        }
        
        let prestate = snapshot_prestate();

        // Get the tag value
        let tag_value = TAGS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&tag_id).map(|tag| tag.value.clone())
        }).unwrap();

        let result = if tag_request.add {
            // Add tag to resource
            add_tag_to_resource(&resource_id, &tag_value)
        } else {
            // Remove tag from resource
            remove_tag_from_resource(&resource_id, &tag_value)
        };
        
        match result {
            Ok(_) => {
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
                        tag: TAGS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&tag_id).cloned()),
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