// src/rest/api_keys/handler.rs

pub mod apikeys_handlers {
    use crate::{
        core::{api::{permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_api_key, generate_uuidv4, mark_claimed_uuid}}, state::{api_keys::{state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyIDList, ApiKeyValue}}, drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::{IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{api_keys::types::{ApiKeyFE, CreateApiKeyRequestBody, CreateApiKeyResponse, DeleteApiKeyRequestBody, DeleteApiKeyResponse, DeletedApiKeyData, ErrorResponse, GetApiKeyResponse, ListApiKeysResponse, UpdateApiKeyRequestBody, UpdateApiKeyResponse}, auth::{authenticate_request, create_auth_error_response}}, 
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_apikey_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {


        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };


       // Get the requested API key ID from params
        let requested_id = ApiKeyID(params.get("api_key_id").unwrap().to_string());

        // Get the requested API key
        let api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&requested_id).map(|key| key.clone())
        });

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        let is_own_key = match &api_key {
            Some(key) => requester_api_key.user_id == key.user_id,
            None => false
        };

        // Check system permissions if not owner or own key
        if !is_owner && !is_own_key {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::ApiKeys),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::ApiKey(requested_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        // let prestate = snapshot_prestate();

 
        match api_key {
            Some(key) => {
                // snapshot_poststate(prestate, Some(
                //     format!(
                //         "{}: Get API Key {}", 
                //         requester_api_key.user_id,
                //         requested_id
                //     ).to_string())
                // );
                let redacted_key = key.cast_fe(&requester_api_key.user_id);
                create_response(
                    StatusCode::OK,
                    GetApiKeyResponse::ok(&redacted_key).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "API key not found".to_string()).encode()
            ),
        }
    }

    pub async fn list_apikeys_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {

        debug_log!("Incoming request: {}", request.url());

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Get the requested user_id from params
        let requested_user_id = UserID(params.get("user_id").unwrap().to_string());

        // Check authorization:
        // 1. The requester's API key must belong to the owner
        // 2. Or the requester must be requesting their own API keys
        // 3. Or the requester must have View permission on the API keys table
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        let is_own_keys = requester_api_key.user_id == requested_user_id;

        if !is_owner && !is_own_keys {
            let resource_id = SystemResourceID::Table(SystemTableEnum::ApiKeys);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        // let prestate = snapshot_prestate();

        // Get the list of API key IDs for the user
        let api_key_ids = USERS_APIKEYS_HASHTABLE.with(|store| {
            store.borrow().get(&requested_user_id).map(|data| data.clone())
        });

        match api_key_ids {
            Some(ids) => {
                // Get full API key details for each ID and convert to hidden version
                let api_keys: Vec<ApiKeyFE> = APIKEYS_BY_ID_HASHTABLE.with(|store| {
                    let store = store.borrow();
                    ids.iter()
                        .filter_map(|id| store.get(id))
                        .map(|key| key.clone().cast_fe(&requester_api_key.user_id))
                        .collect()
                });
                let redacted_api_keys = api_keys.clone().iter().map(|key| key.redacted(&requester_api_key.user_id)).collect();

                // snapshot_poststate(prestate, Some(
                //     format!("{}: List API Keys", requester_api_key.user_id).to_string())
                // );

                create_response(
                    StatusCode::OK,
                    ListApiKeysResponse::ok(&redacted_api_keys).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "No API keys found for user".to_string()).encode()
            ),
        }
    }

    pub async fn create_apikey_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateApiKeyRequestBody>(body).unwrap();
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400,
                    format!("Validation error for field '{}': {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        // Determine what user_id to use for the new key
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
                    
        // Check system permission to create if not owner
        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::ApiKeys);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Create) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // If owner and user_id provided in request, use that. Otherwise use requester's user_id
        let key_user_id = if is_owner && create_req.user_id.is_some() {
            UserID(create_req.user_id.unwrap())
        } else {
            requester_api_key.user_id.clone()
        };

        let unique_id = match create_req.id {
            Some(id) => ApiKeyID(id.to_string()),
            None => ApiKeyID(generate_uuidv4(IDPrefix::ApiKey)),
        };

        // Generate new API key with proper user_id
        let new_api_key = ApiKey {
            id: unique_id.clone(),
            value: ApiKeyValue(generate_api_key()),
            user_id: key_user_id, 
            name: create_req.name,
            private_note: create_req.private_note,
            created_at: ic_cdk::api::time() / 1_000_000,
            begins_at: create_req.begins_at.unwrap_or(0),
            expires_at: create_req.expires_at.unwrap_or(-1),
            is_revoked: false,
            labels: vec![],
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };
        mark_claimed_uuid(&unique_id.to_string());

        // Update all three hashtables
        
        // 1. Add to APIKEYS_BY_VALUE_HASHTABLE
        APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().insert(new_api_key.value.clone(), new_api_key.id.clone());
        });

        // 2. Add to APIKEYS_BY_ID_HASHTABLE
        APIKEYS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(new_api_key.id.clone(), new_api_key.clone());
        });

        // 3. Add to USERS_APIKEYS_HASHTABLE
        USERS_APIKEYS_HASHTABLE.with(|store| {
            let mut store_mut = store.borrow_mut();
            
            // Check if the user already has API keys
            if let Some(mut existing_list) = store_mut.get(&new_api_key.user_id) {
                // Clone and modify the existing list
                let mut updated_list = existing_list.clone();
                updated_list.add(new_api_key.id.clone());
                
                // Insert the updated list
                store_mut.insert(new_api_key.user_id.clone(), updated_list);
            } else {
                // Create new list with this key
                let new_list = ApiKeyIDList::with_key(new_api_key.id.clone());
                store_mut.insert(new_api_key.user_id.clone(), new_list);
            }
        });

        update_external_id_mapping(
            None,
            Some(new_api_key.external_id.clone().unwrap()),
            Some(new_api_key.id.to_string()),
        );

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create API Key {}", 
                requester_api_key.user_id,
                new_api_key.id
            ).to_string())
        );

        let redacted_key = new_api_key.clone().cast_fe(&requester_api_key.user_id);

        create_response(
            StatusCode::OK,
            CreateApiKeyResponse::ok(&redacted_key).encode()
        )
    }

    pub async fn update_apikey_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateApiKeyRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400,
                    format!("Validation error for field '{}': {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        // Get the API key to update
        let api_key_id = ApiKeyID(update_req.id);
        let mut api_key = match APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&api_key_id).map(|key| key.clone())) {
            Some(key) => key,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "API key not found".to_string()).encode()
            ),
        };
        let old_external_id = api_key.external_id.clone();
        let old_internal_id = Some(api_key.id.to_string());

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        let is_own_key = requester_api_key.user_id == api_key.user_id;

        // Check system permission to update if not owner or own key
        if !is_owner && !is_own_key {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::ApiKeys),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::ApiKey(api_key.id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // Update only the fields that were provided
        if let Some(name) = update_req.name {
            api_key.name = name;
        }
        if let Some(private_note) = update_req.private_note {
            api_key.private_note = Some(private_note);
        }
        if let Some(begins_at) = update_req.begins_at {
            api_key.begins_at = begins_at;
        }
        if let Some(expires_at) = update_req.expires_at {
            api_key.expires_at = expires_at;
        }
        if let Some(is_revoked) = update_req.is_revoked {
            api_key.is_revoked = is_revoked;
        }

        if let Some(external_id) = update_req.external_id.clone() {
            api_key.external_id = Some(ExternalID(external_id.clone()));
        }
        if let Some(external_payload) = update_req.external_payload.clone() {
            api_key.external_payload = Some(ExternalPayload(external_payload));
        }

        // Update the API key in APIKEYS_BY_ID_HASHTABLE
        APIKEYS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(api_key.id.clone(), api_key.clone());
        });

        // Get the updated API key
        let updated_api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&api_key.id.clone()).map(|key| key.clone())
        });

        update_external_id_mapping(
            old_external_id,
            Some(ExternalID(update_req.external_id.clone().unwrap_or("".to_string()))),
            old_internal_id
        );

        match updated_api_key {
            Some(key) => {
                snapshot_poststate(prestate, Some(
                    format!(
                        "{}: Update API Key {}", 
                        requester_api_key.user_id,
                        api_key.id
                    ).to_string())
                );
                let redacted_key = key.clone().cast_fe(&requester_api_key.user_id);
                create_response(
                    StatusCode::OK,
                    UpdateApiKeyResponse::ok(&redacted_key).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "API key not found".to_string()).encode()
            ),
        }

    }

    pub async fn delete_apikey_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {

        debug_log!("Incoming request: {}", request.url());

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Parse request body
        let body: &[u8] = request.body();
        
        debug_log!("Incoming request body: {}", String::from_utf8_lossy(request.body()));
        let delete_request = match serde_json::from_slice::<DeleteApiKeyRequestBody>(body) {
            Ok(req) => req,
            Err(_) => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid request format".to_string()).encode()
                )
            }
        };

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400,
                    format!("Validation error for field '{}': {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

       // Get the API key to be deleted
        let api_key_to_delete = APIKEYS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&ApiKeyID(delete_request.id.to_string())).map(|key| key.clone())
        });

        let api_key = match api_key_to_delete {
            Some(key) => key,
            None => {
                return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "API key not found".to_string()).encode()
                )
            }
        };
        let old_external_id = api_key.external_id.clone();
        let old_internal_id = api_key.id.to_string();

        // Check authorization:
        // 1. The requester's API key must belong to the owner
        // 2. Or the requester must be deleting their own API key
        // 3. Or the requester must have Delete permission on this API key record
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        let is_own_key = requester_api_key.user_id == api_key.user_id;

        if !is_owner && !is_own_key {
            let table_permission = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::ApiKeys),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::ApiKey(api_key.id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) && !table_permission.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();
        
        // 1. Remove from APIKEYS_BY_VALUE_HASHTABLE
        APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&api_key.value);
        });

        // 2. Remove from APIKEYS_BY_ID_HASHTABLE
        APIKEYS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&api_key.id);
        });

        // 3. Remove from USERS_APIKEYS_HASHTABLE
        USERS_APIKEYS_HASHTABLE.with(|store| {
            let mut store_mut = store.borrow_mut();
            
            // Check if the user has any API keys
            if let Some(existing_list) = store_mut.get(&api_key.user_id) {
                // Clone and modify the list
                let mut updated_list = existing_list.clone();
                updated_list.remove(&api_key.id);
                
                // If this was the last API key for the user, remove the user entry
                if updated_list.is_empty() {
                    store_mut.remove(&api_key.user_id);
                } else {
                    // Otherwise update with the new list
                    store_mut.insert(api_key.user_id.clone(), updated_list);
                }
            }
        });

        update_external_id_mapping(
            old_external_id,
            None,
            Some(old_internal_id),
        );

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete API Key {}", 
                requester_api_key.user_id,
                api_key.id
            ).to_string())
        );

        // Return success response
        create_response(
            StatusCode::OK,
            DeleteApiKeyResponse::ok(&DeletedApiKeyData {
                id: delete_request.id,
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