// src/rest/webhooks/handler.rs


pub mod webhooks_handlers {
    use std::str::FromStr;

    use crate::{
        core::{
            api::{permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}},
            state::{drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, webhooks::{
                state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE, WEBHOOKS_BY_TIME_LIST}, types::{Webhook, WebhookAltIndexID, WebhookEventLabel, WebhookID}
            }}, types::IDPrefix
        },
        debug_log,
        rest::{
            auth::{authenticate_request, create_auth_error_response}, webhooks::types::{
                CreateWebhookRequestBody, CreateWebhookResponse, DeleteWebhookRequest, DeleteWebhookResponse, DeletedWebhookData, ErrorResponse, GetWebhookResponse, ListWebhooksRequestBody, ListWebhooksResponse, ListWebhooksResponseData, SortDirection, UpdateWebhookRequestBody, UpdateWebhookResponse
            }
        },
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub async fn get_webhook_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };


        // Get webhook ID from params
        let webhook_id = WebhookID(params.get("webhook_id").unwrap().to_string());


        // Only owner can access webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Webhook(webhook_id.to_string()));
            let record_permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Webhooks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !record_permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        // Get the webhook
        let webhook = WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&webhook_id).cloned()
        });


        match webhook {
            Some(hook) => create_response(
                StatusCode::OK,
                GetWebhookResponse::ok(&hook.cast_fe(&requester_api_key.user_id)).encode()
            ),
            None => create_response(
                StatusCode::NOT_FOUND, 
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_webhooks_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Only owner can access webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Groups); // Using Groups since webhooks are group-related
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
        let request_body: ListWebhooksRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
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
    
        // Get total count
        let total_count = WEBHOOKS_BY_TIME_LIST.with(|list| list.borrow().len());
    
        // If there are no webhooks, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListWebhooksResponse::ok(&ListWebhooksResponseData {
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
    
        // Get webhooks with pagination and filtering
        let mut filtered_webhooks = Vec::new();
        let mut processed_count = 0;
    
        WEBHOOKS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            WEBHOOKS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        // Newest first
                        let mut current_idx = start_index;
                        while filtered_webhooks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(webhook) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() || webhook.event.to_string().contains(&request_body.filters) {
                                    filtered_webhooks.push(webhook.clone());
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
                        while filtered_webhooks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(webhook) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() || webhook.event.to_string().contains(&request_body.filters) {
                                    filtered_webhooks.push(webhook.clone());
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
        let response_data = ListWebhooksResponseData {
            items: filtered_webhooks.clone().into_iter().map(|webhook| {
                webhook.cast_fe(&requester_api_key.user_id)
            }).collect(),
            page_size: filtered_webhooks.len(),
            total: total_count,
            cursor_up,
            cursor_down,
        };
    
        create_response(
            StatusCode::OK,
            ListWebhooksResponse::ok(&response_data).encode()
        )
    }

    pub async fn create_webhook_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can manage webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateWebhookRequestBody>(body).unwrap();

        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        // Create new webhook
        let alt_index = WebhookAltIndexID(create_req.alt_index);

        let webhook_id = match create_req.id {
            Some(id) => WebhookID(id.to_string()),
            None => WebhookID(generate_uuidv4(IDPrefix::Webhook)),
        };

        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Webhooks);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Create) {
                return create_auth_error_response();
            }
        }


        let prestate: Option<crate::core::api::replay::diff::EntireState> = snapshot_prestate();
        let event_name = WebhookEventLabel::from_str(&create_req.event).unwrap();
        let webhook = Webhook {
            id: webhook_id.clone(),
            alt_index: alt_index.clone(),
            url: create_req.url,
            event: event_name.clone(),
            signature: create_req.signature.unwrap_or_default(),
            name: create_req.name.unwrap_or(format!("{}@{}", event_name.clone().to_string(), alt_index.0.clone())),
            note: create_req.note,
            active: create_req.active.unwrap_or(true),
            filters: create_req.filters.unwrap_or_default(),
            labels: vec![],
            created_at: ic_cdk::api::time() / 1_000_000,
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };
        update_external_id_mapping(None, webhook.external_id.clone(), Some(webhook.id.clone().to_string()));

        // Update state tables – now storing a Vec<WebhookID> without removing others
        WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            store.entry(alt_index.clone())
                .and_modify(|vec_ids| {
                    if !vec_ids.contains(&webhook_id) {
                        vec_ids.push(webhook_id.clone());
                    }
                })
                .or_insert_with(|| vec![webhook_id.clone()]);
        });

        WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(webhook_id.clone(), webhook.clone());
        });

        WEBHOOKS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(webhook_id.clone());
        });

        mark_claimed_uuid(&webhook_id.clone().to_string());
        
        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Webhook {}", 
                requester_api_key.user_id,
                webhook_id.clone()
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            CreateWebhookResponse::ok(&webhook.cast_fe(&requester_api_key.user_id)).encode()
        )

    }

    pub async fn update_webhook_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can manage webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateWebhookRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        let webhook_id = WebhookID(update_req.id);
        
        // Get existing webhook
        let mut webhook = match WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&webhook_id).cloned()) {
            Some(hook) => hook,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        if !is_owner {
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Webhook(webhook_id.to_string()));
            let record_permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Webhooks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !record_permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // Update fields - ignoring alt_index and event as they cannot be modified
        if let Some(url) = update_req.url {
            webhook.url = url;
        }
        if let Some(signature) = update_req.signature {
            webhook.signature = signature;
        }
        if let Some(name) = update_req.name {
            webhook.name = name;
        }
        if let Some(note) = update_req.note {
            webhook.note = Some(note);
        }
        if let Some(active) = update_req.active {
            webhook.active = active;
        }
        if let Some(filters) = update_req.filters {
            webhook.filters = filters;
        }

        if let Some(external_id) = update_req.external_id.clone() {
            let old_external_id = webhook.external_id.clone();
            let new_external_id = Some(ExternalID(external_id.clone()));
            webhook.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(webhook.id.to_string())
            );
        }
        if let Some(external_payload) = update_req.external_payload.clone() {
            webhook.external_payload = Some(ExternalPayload(external_payload));
        }

        // Update webhook in ID table
        WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(webhook_id.clone(), webhook.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Webhook {}", 
                requester_api_key.user_id,
                webhook_id.clone()
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            UpdateWebhookResponse::ok(&webhook.cast_fe(&requester_api_key.user_id)).encode()
        )

    }


    pub async fn delete_webhook_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteWebhookRequest>(body) {
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

        let webhook_id = WebhookID(delete_request.id.clone());

        // Only owner can manage webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Webhook(webhook_id.to_string()));
            let record_permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Webhooks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !record_permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // Get webhook to delete
        let webhook = match WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&webhook_id).cloned()) {
            Some(hook) => hook,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = webhook.external_id.clone();
        let old_internal_id = Some(webhook.id.clone().to_string());

        // Remove from all hashtables
        WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
            let mut map = store.borrow_mut();
            if let Some(ids) = map.get_mut(&webhook.alt_index) {
                ids.retain(|id| id != &webhook_id);
                if ids.is_empty() {
                    map.remove(&webhook.alt_index);
                }
            }
        });

        WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&webhook_id);
        });

        WEBHOOKS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &webhook_id);
        });

        update_external_id_mapping(old_external_id, None, old_internal_id);

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Webhook {}", 
                requester_api_key.user_id,
                webhook_id.clone()
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            DeleteWebhookResponse::ok(&DeletedWebhookData {
                id: webhook_id,
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