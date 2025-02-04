// src/rest/contacts/handler.rs


pub mod contacts_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, drives::state::state::OWNER_ID}, types::{PublicKeyBLS, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, contacts::types::{ CreateContactResponse, DeleteContactRequest, DeleteContactResponse, DeletedContactData, ErrorResponse, GetContactResponse, ListContactsRequestBody, ListContactsResponse, ListContactsResponseData, UpdateContactRequest, UpdateContactResponse, UpsertContactRequestBody}, webhooks::types::SortDirection}
        
    };
    use crate::core::state::contacts::{
        types::Contact,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_contact_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access contact.private_note
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        // For now, only owner can CRUD contacts - this will change in the future when we add permissions
        if !is_owner {
            return create_auth_error_response();
        }

        // Get contact ID from params
        let contact_id = UserID(params.get("contact_id").unwrap().to_string());

        // Get the contact
        let contact = CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&contact_id).cloned()
        });

        match contact {
            Some(mut contact) => {
                if !is_owner {
                    contact.private_note = None;
                }
                create_response(
                    StatusCode::OK,
                    GetContactResponse::ok(&contact).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND, 
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub fn list_contacts_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Only owner can access webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        // For now, only owner can CRUD contacts - this will change in the future when we add permissions
        if !is_owner {
            return create_auth_error_response();
        }
    
        // Parse request body
        let body = request.body();
        let request_body: ListContactsRequestBody = match serde_json::from_slice(body) {
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
        let total_count = CONTACTS_BY_TIME_LIST.with(|list| list.borrow().len());
    
        // If there are no contacts, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListContactsResponse::ok(&ListContactsResponseData {
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
        let mut filtered_contacts = Vec::new();
        let mut processed_count = 0;
    
        CONTACTS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            CONTACTS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        // Newest first
                        let mut current_idx = start_index;
                        while filtered_contacts.len() < request_body.page_size && current_idx < total_count {
                            if let Some(contact) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() {
                                    filtered_contacts.push(contact.clone());
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
                        while filtered_contacts.len() < request_body.page_size && current_idx < total_count {
                            if let Some(contact) = id_store.get(&time_index[current_idx]) {
                                if request_body.filters.is_empty() {
                                    filtered_contacts.push(contact.clone());
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
        let response_data = ListContactsResponseData {
            items: filtered_contacts.clone(),
            page_size: filtered_contacts.len(),
            total: total_count,
            cursor_up,
            cursor_down,
        };
    
        create_response(
            StatusCode::OK,
            ListContactsResponse::ok(&response_data).encode()
        )
    }

    pub fn upsert_contact_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
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

        if let Ok(req) = serde_json::from_slice::<UpsertContactRequestBody>(body) {
            match req {
                UpsertContactRequestBody::Update(update_req) => {

                    let contact_id = UserID(update_req.id);
                    
                    // Get existing webhook
                    let mut contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&contact_id).cloned()) {
                        Some(contact) => contact,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };

                    // Update fields - ignoring alt_index and event as they cannot be modified
                    if let Some(nickname) = update_req.nickname {
                        contact.nickname = nickname;
                    }
                    if let Some(public_note) = update_req.public_note {
                        contact.public_note = public_note;
                    }
                    if let Some(private_note) = update_req.private_note {
                        if is_owner {
                            contact.private_note = Some(private_note);
                        }
                    }
                    if let Some(evm_public_address) = update_req.evm_public_address {
                        contact.evm_public_address = evm_public_address;
                    }
                    if let Some(icp_principal) = update_req.icp_principal {
                        contact.icp_principal = PublicKeyBLS(icp_principal);
                    }

                    CONTACTS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact_id.clone(), contact.clone());
                    });

                    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact.icp_principal.clone().to_string(), contact_id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateContactResponse::ok(&contact).encode()
                    )
                },
                UpsertContactRequestBody::Create(create_req) => {

                    // Create new webhook
                    let contact_id = UserID(generate_unique_id("UserID", ""));
                    let contact = Contact {
                        id: contact_id.clone(),
                        nickname: create_req.nickname,
                        public_note: create_req.public_note.unwrap_or_default(),
                        private_note: Some(create_req.private_note.unwrap_or_default()),
                        evm_public_address: create_req.evm_public_address.unwrap_or_default(),
                        icp_principal: PublicKeyBLS(create_req.icp_principal),
                        teams: [].to_vec()
                    };

                    CONTACTS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact_id.clone(), contact.clone());
                    });

                    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact.icp_principal.clone().to_string(), contact_id.clone());
                    });

                    CONTACTS_BY_TIME_LIST.with(|store| {
                        store.borrow_mut().push(contact_id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        CreateContactResponse::ok(&contact).encode()
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

    pub fn delete_contact_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
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
        let delete_request = match serde_json::from_slice::<DeleteContactRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        let contact_id = delete_request.id.clone();

        CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&contact_id);
        });

        CONTACTS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &contact_id);
        });

        create_response(
            StatusCode::OK,
            DeleteContactResponse::ok(&DeletedContactData {
                id: contact_id,
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