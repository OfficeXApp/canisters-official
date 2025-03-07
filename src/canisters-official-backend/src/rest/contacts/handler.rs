// src/rest/contacts/handler.rs


pub mod contacts_handlers {
    use crate::{
        core::{api::{permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{format_user_id, generate_unique_id}, webhooks::organization::{fire_organization_webhook, get_active_organization_webhooks}}, state::{contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, drives::{state::state::{superswap_userid, update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, team_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::TeamInviteeID}, teams::state::state::TEAMS_BY_ID_HASHTABLE, webhooks::types::WebhookEventLabel}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, contacts::types::{ CreateContactResponse, DeleteContactRequest, DeleteContactResponse, DeletedContactData, ErrorResponse, GetContactResponse, ListContactsRequestBody, ListContactsResponse, ListContactsResponseData, RedeemContactRequestBody, UpdateContactRequest, UpdateContactResponse, UpsertContactRequestBody}, webhooks::types::SortDirection}
        
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

    pub async fn get_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access contact.private_note
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Get contact ID from params
        let contact_id = UserID(params.get("contact_id").unwrap().to_string());

        // Get the contact
        let contact = CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&contact_id).cloned()
        });

        // Check permissions if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Contacts),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(contact_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        // let prestate = snapshot_prestate();

        match contact {
            Some(mut contact) => {
                if !is_owner {
                    contact.private_note = None;
                }
                // snapshot_poststate(prestate, Some(
                //     format!(
                //         "{}: Get Contact {}", 
                //         requester_api_key.user_id,
                //         contact.id
                //     ).to_string())
                // );
                
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

    pub async fn list_contacts_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Only owner can access webhooks
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        // Check table-level permissions if not owner
        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Contacts);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        // let prestate = snapshot_prestate();
    
        // Parse request body
        let body = request.body();
        let request_body: ListContactsRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
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

        // snapshot_poststate(prestate, Some(
        //     format!(
        //         "{}: List Contacts", 
        //         requester_api_key.user_id
        //     ).to_string())   
        // );
    
        create_response(
            StatusCode::OK,
            ListContactsResponse::ok(&response_data).encode()
        )
    }

    pub async fn upsert_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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

        if let Ok(req) = serde_json::from_slice::<UpsertContactRequestBody>(body) {

            if let Err(validation_error) = req.validate_body() {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(
                        400, 
                        format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                    ).encode()
                );
            }

            match req {
                UpsertContactRequestBody::Update(update_req) => {

                    let contact_id = update_req.id;
                    
                    // Get existing contact
                    let mut contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&contact_id).cloned()) {
                        Some(contact) => contact,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };

                    let old_external_id = contact.external_id.clone();
                    let old_internal_id = Some(contact.id.clone().to_string());

                    // Check update permission if not owner
                    if !is_owner {
                        let table_permissions = check_system_permissions(
                            SystemResourceID::Table(SystemTableEnum::Contacts),
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(contact_id.to_string()));
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Update) && !table_permissions.contains(&SystemPermissionType::Update) {
                            return create_auth_error_response();
                        }
                    }

                    let prestate = snapshot_prestate();

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
                        contact.icp_principal = ICPPrincipalString(PublicKeyICP(icp_principal));
                    }
                    if let Some(seed_phrase) = update_req.seed_phrase {
                        contact.seed_phrase = Some(seed_phrase);
                    }

                    if let Some(external_id) = update_req.external_id.clone() {
                        contact.external_id = Some(ExternalID(external_id));
                    }
                    if let Some(external_payload) = update_req.external_payload.clone() {
                        contact.external_payload = Some(ExternalPayload(external_payload));
                    }

                    CONTACTS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact_id.clone(), contact.clone());
                    });

                    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact.icp_principal.clone(), contact_id.clone());
                    });

                    update_external_id_mapping(
                        old_external_id,
                        Some(ExternalID(update_req.external_payload.unwrap_or("".to_string()))),
                        old_internal_id
                    );

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Update Contact {}", 
                            requester_api_key.user_id,
                            contact.id
                        ).to_string())
                    );

                    create_response(
                        StatusCode::OK,
                        UpdateContactResponse::ok(&contact).encode()
                    )
                },
                UpsertContactRequestBody::Create(create_req) => {

                    // Check create permission if not owner
                    if !is_owner {
                        let resource_id = SystemResourceID::Table(SystemTableEnum::Contacts);
                        let permissions = check_system_permissions(
                            resource_id,
                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                        );
                        
                        if !permissions.contains(&SystemPermissionType::Create) {
                            return create_auth_error_response();
                        }
                    }

                    let prestate = snapshot_prestate();

                    // Create new webhook
                    let contact_id = format_user_id(&create_req.icp_principal.clone());
                    let contact = Contact {
                        id: contact_id.clone(),
                        nickname: create_req.nickname,
                        public_note: create_req.public_note.unwrap_or_default(),
                        private_note: Some(create_req.private_note.unwrap_or_default()),
                        evm_public_address: create_req.evm_public_address.unwrap_or_default(),
                        icp_principal: ICPPrincipalString(PublicKeyICP(create_req.icp_principal)),
                        seed_phrase: Some(create_req.seed_phrase.unwrap_or_default()),
                        teams: [].to_vec(),
                        tags: vec![],
                        past_user_ids: [].to_vec(),
                        external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
                        external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
                        from_placeholder_user_id: create_req.is_placeholder.and_then(|is_placeholder| {
                            if is_placeholder {
                                Some(contact_id.clone())
                            } else {
                                None
                            }
                        }),
                        redeem_token: create_req.is_placeholder.and_then(|is_placeholder| {
                            if is_placeholder {
                                Some(generate_unique_id(IDPrefix::RedeemToken, ""))
                            } else {
                                None
                            }
                        }),
                    };

                    CONTACTS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact_id.clone(), contact.clone());
                    });

                    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(contact.icp_principal.clone(), contact_id.clone());
                    });

                    CONTACTS_BY_TIME_LIST.with(|store| {
                        store.borrow_mut().push(contact_id.clone());
                    });

                    update_external_id_mapping(None, contact.external_id.clone(), Some(contact_id.to_string()));

                    snapshot_poststate(prestate, Some(
                        format!(
                            "{}: Create Contact {}", 
                            requester_api_key.user_id,
                            contact.id
                        ).to_string())
                    );

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

    pub async fn delete_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        if !is_owner {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteContactRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        // Validate request body
        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        let contact_id = delete_request.id.clone();

        let contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&contact_id).cloned()) {
            Some(contact) => contact,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = contact.external_id.clone();
        
        // Check delete permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Contacts),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(contact_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&UserID(contact_id.to_string()));
        });

        CONTACTS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().retain(|id| id != &UserID(contact_id.to_string()));
        });

        // Get and remove user's invites
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            if let Some(invite_ids) = store.borrow_mut().remove(&&TeamInviteeID::User(contact_id.clone())) {
                // Remove each invite from invites hashtable
                INVITES_BY_ID_HASHTABLE.with(|invites_store| {
                    let mut store = invites_store.borrow_mut();
                    for invite_id in invite_ids {
                        if let Some(invite) = store.remove(&invite_id) {
                            // Remove user from team if they were part of it
                            TEAMS_BY_ID_HASHTABLE.with(|teams_store| {
                                if let Some(mut team) = teams_store.borrow_mut().get_mut(&invite.team_id) {
                                    team.member_invites.retain(|member_invite_id| member_invite_id != &invite_id);
                                    team.admin_invites.retain(|admin_invite_id| admin_invite_id != &invite_id);
                                }
                            });
                        }
                    }
                });
            }
        });

        update_external_id_mapping(old_external_id, None, Some(contact_id.to_string()));

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Contact {}", 
                requester_api_key.user_id,
                contact_id
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            DeleteContactResponse::ok(&DeletedContactData {
                id: contact_id.clone(),
                deleted: true
            }).encode()
        )
    }

    pub async fn redeem_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        let prestate = snapshot_prestate();

        // Parse request body
        let body: &[u8] = request.body();
        let redeem_request = match serde_json::from_slice::<RedeemContactRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        // Validate request body
        if let Err(validation_error) = redeem_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        let current_user_id = UserID(redeem_request.current_user_id.clone());
        let new_user_id = UserID(redeem_request.new_user_id.clone());
        let redeem_token = redeem_request.redeem_token.clone();

        // Check for existence of current user contact and redeem token match
        let current_contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&current_user_id).cloned()) {
            Some(contact) => contact,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        // throw error if redeem token does not match
        if current_contact.redeem_token != Some(redeem_token.clone()) {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Redeem token does not match".to_string()).encode()
            );
        }

        // superswap the user_ids with superswap_userid which returns the number of records updated or error
        match superswap_userid(current_user_id.clone(), new_user_id.clone()) {
            Ok(update_count) => {
                // Update the redeem token to None
                CONTACTS_BY_ID_HASHTABLE.with(|store| {
                    if let Some(contact) = store.borrow_mut().get_mut(&current_user_id) {
                        contact.redeem_token = None;
                    }
                });

                let active_webhooks = get_active_organization_webhooks(
                    WebhookEventLabel::OrganizationSuperswapUser
                );

                // Fire organization webhook
                fire_organization_webhook(
                    WebhookEventLabel::OrganizationSuperswapUser,
                    active_webhooks,
                    Some(current_user_id.clone()),
                    Some(new_user_id.clone()),
                    Some(format!("Redeem Contact - superswap {} to {}, updated {} records", current_user_id, new_user_id, update_count))
                );

                snapshot_poststate(prestate, Some(
                    format!(
                        "{}: Redeem Contact - superswap {} to {}, updated {} records", 
                        requester_api_key.user_id,
                        current_user_id,
                        new_user_id,
                        update_count
                    ).to_string())
                );

                create_response(
                    StatusCode::OK,
                    UpdateContactResponse::ok(&current_contact).encode()
                )
            },
            Err(err) => {
                create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err).encode()
                )
            }
        }

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