// src/rest/contacts/handler.rs


pub mod contacts_handlers {
    use crate::{
        core::{api::{permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{format_user_id, generate_api_key, generate_uuidv4, mark_claimed_uuid}, webhooks::organization::{fire_superswap_user_webhook, get_superswap_user_webhooks}}, state::{api_keys::{state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyIDList, ApiKeyValue}}, contacts::state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, drives::{state::state::{superswap_userid, update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, group_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::{GroupInvite, GroupInviteID, GroupInviteeID, GroupRole}}, groups::state::state::{DEFAULT_EVERYONE_GROUP, GROUPS_BY_ID_HASHTABLE}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, webhooks::types::WebhookEventLabel}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, contacts::types::{ CreateContactRequestBody, CreateContactResponse, DeleteContactRequest, DeleteContactResponse, DeletedContactData, ErrorResponse, GetContactResponse, ListContactsRequestBody, ListContactsResponse, ListContactsResponseData, RedeemContactRequestBody, RedeemContactResponse, RedeemContactResponseBody, UpdateContactRequest, UpdateContactRequestBody, UpdateContactResponse}, webhooks::types::SortDirection}
        
    };
    use crate::core::state::contacts::{
        types::Contact,
    };
    use url::Url;
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Get contact ID from params
        let contact_id = UserID(params.get("contact_id").unwrap().to_string());

        // Get the contact
        let contact = CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&contact_id).map(|data| data.clone())
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
                let cast_fe_contact = contact.clone().cast_fe(&requester_api_key.user_id);
                create_response(
                    StatusCode::OK,
                    GetContactResponse::ok(&cast_fe_contact).encode()
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
        
        // Check if the requester is the owner (who has full access)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
        
        // Check table-level permissions
        let has_table_permission = if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Contacts);
            debug_log!("Checking permissions for resource: {:?} for user {}", resource_id, requester_api_key.user_id);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            debug_log!("Permissions: {:?}", permissions);
            debug_log!("Matching against {:?}", SystemPermissionType::View);
            permissions.contains(&SystemPermissionType::View)
        } else {
            true
        };

        debug_log!("has_table_permission: {}", has_table_permission);
    
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
        let total_count = CONTACTS_BY_TIME_LIST.with(|list| list.borrow().len() as usize);
    
        // If there are no contacts, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListContactsResponse::ok(&ListContactsResponseData {
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
    
        // Get contacts with pagination and filtering, applying permission checks
        let mut filtered_contacts = Vec::new();
        let mut processed_count = 0;
        let mut end_index = start_index;  // Track where we ended for cursor calculation
        let mut total_count_to_return = 0; // Will use this for the response
    
        // If user is owner or has table access, they get the actual total count
        // Otherwise, we'll only report the batch size + 1 (if there are more)
        if is_owner || has_table_permission {
            total_count_to_return = total_count;
        }
    
        CONTACTS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            CONTACTS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();
                
                match request_body.direction {
                    SortDirection::Desc => {
                        // Newest first
                        let mut current_idx = start_index;
                        while filtered_contacts.len() < request_body.page_size && current_idx < total_count {
                            if let Some(contact) = id_store.get(&time_index.get(current_idx as u64).unwrap_or_else(|| panic!("Index out of bounds"))) {
                                let can_view = is_owner || has_table_permission || {
                                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(contact.id.to_string()));
                                    let permissions = check_system_permissions(
                                        resource_id,
                                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                                    );
                                    permissions.contains(&SystemPermissionType::View)
                                };
    
                                if can_view {
                                    filtered_contacts.push(contact.clone());
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
                        while filtered_contacts.len() < request_body.page_size && current_idx < total_count {
                            if let Some(contact) = id_store.get(&time_index.get(current_idx as u64).unwrap_or_else(|| panic!("Index out of bounds"))) {
                                let can_view = is_owner || has_table_permission || {
                                    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(contact.id.to_string()));
                                    let permissions = check_system_permissions(
                                        resource_id,
                                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                                    );
                                    permissions.contains(&SystemPermissionType::View)
                                };
    
                                if can_view {
                                    filtered_contacts.push(contact.clone());
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
        let next_cursor = if filtered_contacts.len() >= request_body.page_size {
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
                total_count_to_return = filtered_contacts.len() + 1;
            } else {
                // Otherwise, just return the batch size
                total_count_to_return = filtered_contacts.len();
            }
        }
    
        // Create response
        let response_data = ListContactsResponseData {
            items: filtered_contacts.clone().into_iter().map(|contact| {
                contact.cast_fe(&requester_api_key.user_id)
            }).collect(),
            page_size: filtered_contacts.len(),
            total: total_count_to_return,
            direction: request_body.direction,
            cursor: next_cursor,
        };
    
        create_response(
            StatusCode::OK,
            ListContactsResponse::ok(&response_data).encode()
        )
    }

    pub async fn create_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateContactRequestBody>(body).unwrap();

        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }
        
        // Check create permission if not owner
        if !is_owner {

            debug_log!("Checking create permission on resource {} for user {}", SystemTableEnum::Contacts, requester_api_key.user_id);

            let resource_id = SystemResourceID::Table(SystemTableEnum::Contacts);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            debug_log!("Permissions: {:?}", permissions);
            
            if !permissions.contains(&SystemPermissionType::Create) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        let contact_id = format_user_id(&create_req.icp_principal.clone());
        let contact = Contact {
            id: contact_id.clone(),
            name: create_req.name,
            email: create_req.email,
            avatar: create_req.avatar,
            notifications_url: create_req.notifications_url,
            public_note: create_req.public_note,
            private_note: Some(create_req.private_note.unwrap_or_default()),
            evm_public_address: create_req.evm_public_address.unwrap_or_default(),
            icp_principal: ICPPrincipalString(PublicKeyICP(create_req.icp_principal)),
            seed_phrase: Some(create_req.seed_phrase.unwrap_or_default()),
            groups: [].to_vec(),
            labels: vec![],
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
            redeem_code: create_req.is_placeholder.and_then(|is_placeholder| {
                if is_placeholder {
                    Some(generate_uuidv4(IDPrefix::RedeemCode))
                    // unnessary to mark_claimed_uuid since this we generate the redeem code on the fly (not from the client)
                } else {
                    None
                }
            }),
            created_at: ic_cdk::api::time() / 1_000_000,
            last_online_ms: 0,
        };

        CONTACTS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(contact_id.clone(), contact.clone());
        });

        CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| {
            store.borrow_mut().insert(contact.icp_principal.clone(), contact_id.clone());
        });

        CONTACTS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(&contact_id);
        });

        update_external_id_mapping(None, contact.external_id.clone(), Some(contact_id.to_string()));

        mark_claimed_uuid(&contact_id.to_string());

        // Add the contact to the default "Everyone" group if it exists
        let default_group_id = DEFAULT_EVERYONE_GROUP.with(|group_id| group_id.borrow().clone());
        let owner_id = OWNER_ID.with(|owner_id| owner_id.borrow().get().clone());
        
        // Check if the default group still exists
        let group_exists = GROUPS_BY_ID_HASHTABLE.with(|groups| {
            groups.borrow().contains_key(&default_group_id)
        });
        
        if group_exists {
            // Create a new group invite for the contact
            let invite_id = GroupInviteID(generate_uuidv4(IDPrefix::GroupInvite));
            let current_time = ic_cdk::api::time() / 1_000_000;
            
            let group_invite = GroupInvite {
                id: invite_id.clone(),
                group_id: default_group_id.clone(),
                inviter_id: owner_id.clone(),
                invitee_id: GroupInviteeID::User(contact_id.clone()),
                role: GroupRole::Member,
                note: format!("Auto-added to default group upon contact creation"),
                active_from: current_time,
                expires_at: -1, // Never expires
                created_at: current_time,
                last_modified_at: current_time,
                redeem_code: None,
                from_placeholder_invitee: None,
                labels: Vec::new(),
                external_id: None,
                external_payload: None,
            };
            
            // Store the invite in hashtable
            INVITES_BY_ID_HASHTABLE.with(|invites| {
                invites.borrow_mut().insert(invite_id.clone(), group_invite.clone());
            });
            
            // Add to user's invites list
            USERS_INVITES_LIST_HASHTABLE.with(|users_invites| {
                let mut users_invites_ref = users_invites.borrow_mut();
                let user_invites = users_invites_ref.entry(GroupInviteeID::User(contact_id.clone())).or_insert_with(Vec::new);
                user_invites.push(invite_id.clone());
            });
            
            // Update the group's member_invites list
            GROUPS_BY_ID_HASHTABLE.with(|groups| {
                if let Some(group) = groups.borrow_mut().get_mut(&default_group_id) {
                    group.member_invites.push(invite_id);
                    group.last_modified_at = current_time;
                }
            });
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Contact {}", 
                requester_api_key.user_id,
                contact.id
            ).to_string())
        );

        let cast_fe_contact = contact.clone().cast_fe(&requester_api_key.user_id);

        create_response(
            StatusCode::OK,
            CreateContactResponse::ok(&cast_fe_contact).encode()
        )

    }

    pub async fn update_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
      

        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateContactRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        let contact_id = update_req.id;
                    
        // Get existing contact
        let mut contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&contact_id).map(|data| data.clone())) {
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
            
            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }

        let prestate = snapshot_prestate();

        // Update fields - ignoring alt_index and event as they cannot be modified
        if let Some(name) = update_req.name {
            contact.name = name;
        }
        if let Some(public_note) = update_req.public_note {
            contact.public_note = Some(public_note);
        }
        if let Some(private_note) = update_req.private_note {
            if is_owner {
                contact.private_note = Some(private_note);
            }
        }
        if let Some(email) = update_req.email {
            contact.email = Some(email);
        }
        if let Some(avatar) = update_req.avatar {
            contact.avatar = Some(avatar);
        }
        if let Some(notifications_url) = update_req.notifications_url {
            contact.notifications_url = Some(notifications_url);
        }
        if let Some(evm_public_address) = update_req.evm_public_address {
            contact.evm_public_address = evm_public_address;
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
        let cast_fe_contact = contact.clone().cast_fe(&requester_api_key.user_id);

        create_response(
            StatusCode::OK,
            UpdateContactResponse::ok(&cast_fe_contact).encode()
        )

    }

    pub async fn delete_contact_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());
    

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

        let contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&contact_id).map(|data| data.clone())) {
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
            let mut new_vec = Vec::new();
            let mut time_list = store.borrow_mut();
            
            for i in 0..time_list.len() {
                if let Some(id) = time_list.get(i) {
                    if id != UserID(contact_id.to_string()) {
                        new_vec.push(id.clone());
                    }
                }
            }
            
            // Clear and rebuild the StableVec
            while time_list.len() > 0 {
                time_list.pop();
            }
            
            for id in new_vec {
                time_list.push(&id);
            }
        });

        // Get and remove user's invites
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            if let Some(invite_ids) = store.borrow_mut().remove(&&GroupInviteeID::User(contact_id.clone())) {
                // Remove each invite from invites hashtable
                INVITES_BY_ID_HASHTABLE.with(|invites_store| {
                    let mut store = invites_store.borrow_mut();
                    for invite_id in invite_ids {
                        if let Some(invite) = store.remove(&invite_id) {
                            // Remove user from group if they were part of it
                            GROUPS_BY_ID_HASHTABLE.with(|groups_store| {
                                if let Some(mut group) = groups_store.borrow_mut().get_mut(&invite.group_id) {
                                    group.member_invites.retain(|member_invite_id| member_invite_id != &invite_id);
                                    group.admin_invites.retain(|admin_invite_id| admin_invite_id != &invite_id);
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
        

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

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
        let redeem_code = redeem_request.redeem_code.clone();

        // Check for existence of current user contact and redeem token match
        let current_contact = match CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&current_user_id).map(|data| data.clone())) {
            Some(contact) => contact,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        // throw error if redeem token does not match
        if current_contact.redeem_code != Some(redeem_code.clone()) {
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
                    let mut store_ref = store.borrow_mut();
                    if let Some(mut contact) = store_ref.get(&new_user_id).map(|data| data.clone()) {
                        contact.redeem_code = None;
                        store_ref.insert(new_user_id.clone(), contact);
                    }
                });


                let active_webhooks = get_superswap_user_webhooks(
                    WebhookEventLabel::OrganizationSuperswapUser
                );

                // Fire organization webhook
                fire_superswap_user_webhook(
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

                let cast_fe_contact = current_contact.clone().cast_fe(&requester_api_key.user_id);


                let unique_id = ApiKeyID(generate_uuidv4(IDPrefix::ApiKey));
        
                // Generate new API key with proper user_id
                let new_api_key = ApiKey {
                    id: unique_id.clone(),
                    value: ApiKeyValue(generate_api_key()),
                    user_id: new_user_id.clone(), 
                    name: "Superswap User API Key".to_string(),
                    private_note: Some("Automatically generated API key for superswapped user".to_string()),
                    created_at: ic_cdk::api::time(),
                    begins_at: 0,
                    expires_at: -1,
                    is_revoked: false,
                    labels: vec![],
                    external_id: None,
                    external_payload: None,
                };
                mark_claimed_uuid(&unique_id.to_string());

                // create new api key for this user
                let api_key_value = new_api_key.value.clone();
        
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
                    
                    if let Some(existing_list) = store_mut.get(&new_api_key.user_id) {
                        // Clone and modify the existing list
                        let mut updated_list = existing_list.clone();
                        updated_list.add(new_api_key.id.clone());
                        store_mut.insert(new_api_key.user_id.clone(), updated_list);
                    } else {
                        // Create new list with this key
                        store_mut.insert(new_api_key.user_id.clone(), ApiKeyIDList::with_key(new_api_key.id.clone()));
                    }
                });
                
                create_response(
                    StatusCode::OK,
                    RedeemContactResponse::ok(&RedeemContactResponseBody {
                        contact: cast_fe_contact,
                        api_key: api_key_value,
                    }).encode()
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