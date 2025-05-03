// src/rest/api_keys/handler.rs

pub mod apikeys_handlers {
    use crate::{
        core::{api::uuid::{generate_api_key, generate_uuidv4}, state::{api_keys::{state::state::{APIKEYS_BY_HISTORY, APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, APIKEYS_BY_VALUE_MEMORY_ID, USERS_APIKEYS_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyIDList, ApiKeyValue}}, giftcards_spawnorg::state::state::OWNER_ID}, types::{IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{api_keys::types::{CreateApiKeyRequestBody, CreateApiKeyResponse, DeleteApiKeyRequestBody, DeleteApiKeyResponse, DeletedApiKeyData, ErrorResponse, GetApiKeyResponse, ListApiKeysResponse, SnapshotResponse, StateSnapshot, UpdateApiKeyRequestBody, UpdateApiKeyResponse, UpsertApiKeyRequestBody}, auth::{authenticate_request, create_auth_error_response}}, DEPLOYMENT_STAGE, MEMORY_MANAGER, _DEPLOYMENT_STAGING 
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use ic_stable_structures::StableBTreeMap;
    use matchit::Params;
    use serde::Deserialize;
    use crate::{core::state::giftcards_spawnorg::{state::state::{CANISTER_ID, VERSION, URL_ENDPOINT, DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID, HISTORICAL_GIFTCARDS_SPAWNORGS, DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE, USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE, GIFTCARD_SPAWNORG_BY_ID}, types::{DriveID}}};

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
            store.borrow().get(&requested_id).clone()
        });

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        let is_own_key = match &api_key {
            Some(key) => requester_api_key.user_id == key.user_id,
            None => false
        };

        // Check system permissions if not owner or own key
        if !is_owner && !is_own_key {
            return create_auth_error_response();
        }
 
        match api_key {
            Some(key) => {
               
                create_response(
                    StatusCode::OK,
                    GetApiKeyResponse::ok(&key).encode()
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

        // Disable authentication for now, just for development ease

        // Authenticate request
        // let requester_api_key = match authenticate_request(request) {
        //     Some(key) => key,
        //     None => return create_auth_error_response(),
        // };

        // // Get the requested user_id from params
        // let requested_user_id = UserID(params.get("user_id").unwrap().to_string());

        // // Check authorization:
        // // 1. The requester's API key must belong to the owner
        // // 2. Or the requester must be requesting their own API keys
        // // 3. Or the requester must have View permission on the API keys table
        // let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        // let is_own_keys = requester_api_key.user_id == requested_user_id;

        // if !is_owner && !is_own_keys {
        //     return create_auth_error_response();
        // }

        // // Get the list of API key IDs for the user
        // let api_key_ids = USERS_APIKEYS_HASHTABLE.with(|store| {
        //     store.borrow().get(&requested_user_id).cloned()
        // });

        // just get all api keys, just 
        

        // Get all API keys from history
        let api_keys = APIKEYS_BY_HISTORY.with(|history| {
            let history = history.borrow();
            let api_keys: Vec<ApiKey> = APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let store = store.borrow();
                history.iter()
                    .filter_map(|id| store.get(&id))
                    .map(|key| ApiKey::from(key.clone()))
                    .collect()
            });
            api_keys
        });

        if api_keys.is_empty() {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "No API keys found".to_string()).encode()
            );
        }

        create_response(
            StatusCode::OK,
            ListApiKeysResponse::ok(&api_keys).encode()
        )
    }

    pub async fn upsert_apikey_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();

        if let Ok(req) = serde_json::from_slice::<UpsertApiKeyRequestBody>(body) {

            // Validate request body
            if let Err(validation_error) = req.validate_body() {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(
                        400,
                        format!("Validation error for field '{}': {}", validation_error.field, validation_error.message)
                    ).encode()
                );
            }

            match req {
                UpsertApiKeyRequestBody::Create(create_req) => {
            
                    // Determine what user_id to use for the new key
                    let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
                    
                    // Check system permission to create if not owner
                    if !is_owner {
                        return create_auth_error_response();
                    }

                    // If owner and user_id provided in request, use that. Otherwise use requester's user_id
                    let key_user_id = if is_owner && create_req.user_id.is_some() {
                        UserID(create_req.user_id.unwrap())
                    } else {
                        requester_api_key.user_id.clone()
                    };
            
                    // Generate new API key with proper user_id
                    let new_api_key = ApiKey {
                        id: ApiKeyID(generate_uuidv4(IDPrefix::ApiKey)),
                        value: ApiKeyValue(generate_api_key()),
                        user_id: key_user_id, 
                        name: create_req.name,
                        created_at: ic_cdk::api::time() / 1_000_000,
                        expires_at: create_req.expires_at.unwrap_or(-1),
                        is_revoked: false,
                    };
            
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
                        let mut store_ref = store.borrow_mut();
                        let mut key_list = match store_ref.get(&new_api_key.user_id) {
                            Some(list) => list.clone(),
                            None => ApiKeyIDList::new(),
                        };
                        key_list.keys.push(new_api_key.id.clone()); 
                        store_ref.insert(new_api_key.user_id.clone(), key_list);
                    });

                    create_response(
                        StatusCode::OK,
                        CreateApiKeyResponse::ok(&new_api_key).encode()
                    )  
                },
                UpsertApiKeyRequestBody::Update(update_req) => {
            
                    // Get the API key to update
                    let api_key_id = ApiKeyID(update_req.id);
                    let mut api_key = match APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&api_key_id).clone()) {
                        Some(key) => key,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::err(404, "API key not found".to_string()).encode()
                        ),
                    };

                    let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
                    let is_own_key = requester_api_key.user_id == api_key.user_id;

                    // Check system permission to update if not owner or own key
                    if !is_owner && !is_own_key {
                        return create_auth_error_response();
                    }

                    // Update only the fields that were provided
                    if let Some(name) = update_req.name {
                        api_key.name = name;
                    }
                    if let Some(expires_at) = update_req.expires_at {
                        api_key.expires_at = expires_at;
                    }
                    if let Some(is_revoked) = update_req.is_revoked {
                        api_key.is_revoked = is_revoked;
                    }

            
                    // Update the API key in APIKEYS_BY_ID_HASHTABLE
                    APIKEYS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(api_key.id.clone(), api_key.clone());
                    });

                    // Get the updated API key
                    let updated_api_key = APIKEYS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow().get(&api_key.id.clone()).clone()
                    });


                    match updated_api_key {
                        Some(key) => {
                           
                            create_response(
                                StatusCode::OK,
                                UpdateApiKeyResponse::ok(&key).encode()
                            )
                        },
                        None => create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::err(404, "API key not found".to_string()).encode()
                        ),
                    }
                }
            }
        } else {
            create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            )
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
            store.borrow().get(&ApiKeyID(delete_request.id.to_string())).clone()
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
        // Check authorization:
        // 1. The requester's API key must belong to the owner
        // 2. Or the requester must be deleting their own API key
        // 3. Or the requester must have Delete permission on this API key record
        let owner_id = OWNER_ID.with(|id| (*id.borrow().get()).clone());
        let is_owner = owner_id == requester_api_key.user_id;
        let is_own_key = requester_api_key.user_id == api_key.user_id;

        if !is_owner && !is_own_key {
            return create_auth_error_response();
        }

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
            let mut store_ref = store.borrow_mut();
            if let Some(api_key_ids) = store_ref.get(&api_key.user_id) {
                let mut updated_ids = api_key_ids.clone();
                updated_ids.keys.retain(|id| id != &api_key.id); // Assuming ApiKeyIDList has a retain method
                
                // If this was the last API key for the user, remove the user entry
                if updated_ids.is_empty() { // Assuming ApiKeyIDList has an is_empty method
                    store_ref.remove(&api_key.user_id);
                } else {
                    store_ref.insert(api_key.user_id.clone(), updated_ids);
                }
            }
        });


        // Return success response
        create_response(
            StatusCode::OK,
            DeleteApiKeyResponse::ok(&DeletedApiKeyData {
                id: delete_request.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn snapshot_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Incoming snapshot request: {}", request.url());
    
        if _DEPLOYMENT_STAGING != DEPLOYMENT_STAGE::LocalDevelopment {
            // // Authenticate request
            let requester_api_key = match authenticate_request(request) {
                Some(key) => key,
                None => return create_auth_error_response(),
            };
        
            // Check authorization - only owner can access snapshots
            let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
            if !is_owner {
                return create_auth_error_response();
            }
        }

    
        // Create a snapshot of the entire state
        let state_snapshot = StateSnapshot {
            // System info
            canister_id: CANISTER_ID.with(|id| id.clone()),
            version: VERSION.with(|v| v.borrow().get().clone()),
            owner_id: OWNER_ID.with(|id| id.borrow().get().clone()),
            endpoint_url: URL_ENDPOINT.with(|url| url.borrow().get().clone()),
            
            // API keys state
            apikeys_by_value: APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.clone());
                }
                result
            }),
            apikeys_by_id: APIKEYS_BY_ID_HASHTABLE.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.clone());
                }
                result
            }),
            users_apikeys: USERS_APIKEYS_HASHTABLE.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.keys.clone());
                }
                result
            }),
            apikeys_history: APIKEYS_BY_HISTORY.with(|vec| {
                let borrowed = vec.borrow();
                let mut result = Vec::new();
                for i in 0..borrowed.len() {
                    if let Some(item) = borrowed.get(i) {
                        result.push(item.clone());
                    }
                }
                result
            }),
            
            // GiftcardSpawnOrg state
            deployments_by_giftcard_id: DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.clone());
                }
                result
            }),
            historical_giftcards: HISTORICAL_GIFTCARDS_SPAWNORGS.with(|vec| {
                let borrowed = vec.borrow();
                let mut result = Vec::new();
                for i in 0..borrowed.len() {
                    if let Some(item) = borrowed.get(i) {
                        result.push(item.clone());
                    }
                }
                result
            }),
            drive_to_giftcard_hashtable: DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.clone());
                }
                result
            }),
            user_to_giftcards_hashtable: USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.items.clone());
                }
                result
            }),
            giftcard_by_id: GIFTCARD_SPAWNORG_BY_ID.with(|store| {
                let borrowed = store.borrow();
                let mut result = std::collections::HashMap::new();
                for (k, v) in borrowed.iter() {
                    result.insert(k.clone(), v.clone());
                }
                result
            }),
            
            // Add timestamp
            timestamp_ns: ic_cdk::api::time(),
        };
    
        // Create response
        let response = SnapshotResponse {
            status: "success".to_string(),
            data: state_snapshot,
            timestamp: ic_cdk::api::time() / 1_000_000,
        };
    
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&response).unwrap_or_default()
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