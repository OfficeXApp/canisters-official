// src/rest/group_invites/handler.rs


pub mod group_invites_handlers {
    use crate::{
        core::{api::{permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}, webhooks::group_invites::{fire_group_invite_webhook, get_active_group_invite_webhooks}}, state::{drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, group_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::{GroupInviteID, GroupInviteeID, GroupRole, PlaceholderGroupInviteeID}}, groups::{state::state::{is_user_on_group, GROUPS_BY_ID_HASHTABLE}, types::GroupID}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemResourceID, SystemTableEnum}, webhooks::types::WebhookEventLabel}, types::{IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, group_invites::types::{ CreateGroupInviteRequestBody, CreateGroup_InviteResponse, DeleteGroup_InviteRequest, DeleteGroup_InviteResponse, DeletedGroup_InviteData, ErrorResponse, GetGroup_InviteResponse, ListGroupInvitesRequestBody, ListGroupInvitesResponseData, ListGroup_InvitesResponse, RedeemGroupInviteRequest, RedeemGroupInviteResponseData, UpdateGroupInviteRequestBody, UpdateGroup_InviteRequest, UpdateGroup_InviteResponse}, groups::types::{ListGroupsRequestBody, ListGroupsResponseData}, webhooks::types::{GroupInviteWebhookData, SortDirection}}
        
    };
    use crate::core::state::group_invites::{
        types::GroupInvite,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub async fn get_group_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        let invite_id = GroupInviteID(params.get("invite_id").unwrap().to_string());
        
        let invite = INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&invite_id).cloned()
        });
    
        match invite {
            Some(invite) => {
                // Check if user is authorized (group owner, admin, or invitee)
                let is_authorized = GROUPS_BY_ID_HASHTABLE.with(|store| {
                    if let Some(group) = store.borrow().get(&invite.group_id) {
                        group.owner == requester_api_key.user_id.clone() || 
                        group.admin_invites.contains(&invite.id) ||
                        invite.invitee_id == GroupInviteeID::User(requester_api_key.user_id.clone())
                    } else {
                        false
                    }
                });

                let table_permissions = check_system_permissions(
                    SystemResourceID::Table(SystemTableEnum::Groups),
                    PermissionGranteeID::User(requester_api_key.user_id.clone())
                );
    
                if !is_authorized && !table_permissions.contains(&SystemPermissionType::View) {
                    return create_auth_error_response();
                }
    
                create_response(
                    StatusCode::OK,
                    GetGroup_InviteResponse::ok(&invite.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }
    
    pub async fn list_group_invites_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let query: ListGroupInvitesRequestBody = match serde_json::from_slice(request.body()) {
            Ok(q) => q,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        if let Err(validation_err) = query.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_err.field, validation_err.message)
                ).encode()
            );
        }
    
        let group_id = GroupID(query.group_id.clone());
    
        // Check if the group exists first
        let group_exists = GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().contains_key(&group_id)
        });
        if !group_exists {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(400, "Group not found".to_string()).encode()
            );
        }
    
        // Check if user is owner or admin
        let is_authorized = GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .get(&group_id)
                .map(|group| {
                    group.owner == requester_api_key.user_id.clone() || 
                    group.admin_invites.iter().any(|invite_id| {
                        INVITES_BY_ID_HASHTABLE.with(|invite_store| {
                            invite_store.borrow()
                                .get(invite_id)
                                .map(|invite| invite.invitee_id == GroupInviteeID::User(requester_api_key.user_id.clone()))
                                .unwrap_or(false)
                        })
                    })
                })
                .unwrap_or(false)
        });
    
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
    
        let is_member = is_user_on_group(&requester_api_key.user_id, &group_id).await;
    
        if !is_authorized && !table_permissions.contains(&SystemPermissionType::View) && !is_member  {
            return create_auth_error_response();
        }
    
        let all_invites = GROUPS_BY_ID_HASHTABLE.with(|groups_store| {
            let groups = groups_store.borrow();
            groups.get(&group_id)
                .map(|group| INVITES_BY_ID_HASHTABLE.with(|invite_store| {
                    let invites = invite_store.borrow();
                    group.member_invites.iter()
                        .filter_map(|id| invites.get(id))
                        .cloned()
                        .collect::<Vec<_>>()
                }))
                .unwrap_or_default()
        });
    
        // If there are no invites, return early
        if all_invites.is_empty() {
            return create_response(
                StatusCode::OK,
                ListGroup_InvitesResponse::ok(&ListGroupInvitesResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    direction: query.direction,
                    cursor: None,
                }).encode()
            );
        }
    
        // Determine start position based on cursor
        let start_position = if let Some(cursor_value) = query.cursor {
            match all_invites.iter().position(|i| i.id.0 == cursor_value) {
                Some(pos) => {
                    // If ascending (default), start after the cursor
                    // If descending, start before the cursor
                    match query.direction {
                        SortDirection::Asc => pos + 1,
                        SortDirection::Desc => {
                            if pos > 0 {
                                pos - 1
                            } else {
                                0
                            }
                        }
                    }
                },
                None => 0, // Cursor not found, start from beginning
            }
        } else {
            // No cursor provided, start from beginning or end based on direction
            match query.direction {
                SortDirection::Asc => 0, // Start from beginning
                SortDirection::Desc => {
                    if all_invites.is_empty() {
                        0
                    } else {
                        all_invites.len() - 1 // Start from end
                    }
                }
            }
        };
    
        // Get paginated items based on direction
        let items = match query.direction {
            SortDirection::Asc => {
                all_invites
                    .iter()
                    .skip(start_position)
                    .take(query.page_size)
                    .cloned()
                    .collect::<Vec<_>>()
            },
            SortDirection::Desc => {
                // If descending, we need to collect items in reverse order
                let mut items = Vec::new();
                let mut current_pos = start_position;
                
                while items.len() < query.page_size && current_pos < all_invites.len() {
                    items.push(all_invites[current_pos].clone());
                    if current_pos == 0 {
                        break;
                    }
                    current_pos -= 1;
                }
                
                items
            }
        };
    
        // Determine next cursor
        let next_cursor = if items.is_empty() {
            None
        } else {
            match query.direction {
                SortDirection::Asc => {
                    // For ascending order, use the last item's ID if there might be more
                    if start_position + items.len() < all_invites.len() {
                        items.last().map(|i| i.id.0.clone())
                    } else {
                        None
                    }
                },
                SortDirection::Desc => {
                    // For descending order, use the last accessed item's ID if there might be more
                    if start_position > items.len() {
                        Some(all_invites[start_position - items.len()].id.0.clone())
                    } else {
                        None
                    }
                }
            }
        };
    
        let response_data = ListGroupInvitesResponseData {
            items: items.clone().into_iter().map(|invite| invite.cast_fe(&requester_api_key.user_id)).collect(),
            page_size: query.page_size,
            total: all_invites.len(),
            direction: query.direction,
            cursor: next_cursor,
        };
    
        create_response(
            StatusCode::OK,
            ListGroup_InvitesResponse::ok(&response_data).encode()
        )
    }

    pub async fn create_group_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateGroupInviteRequestBody>(body).unwrap();
        
        if let Err(validation_err) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_err.field, validation_err.message)
                ).encode()
            );
        }

        let group_id = GroupID(create_req.group_id);
        let active_webhooks = get_active_group_invite_webhooks(&group_id, WebhookEventLabel::GroupInviteCreated);

        let before_snap = GroupInviteWebhookData {
            group: GROUPS_BY_ID_HASHTABLE.with(|store| 
                store.borrow().get(&group_id).cloned()
            ),
            group_invite: None,
        };

        // Verify group exists and user has permission
        let group = match GROUPS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&group_id).cloned()) {
            Some(group) => group,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Check if user is authorized (owner or admin)
        let is_authorized = group.owner == requester_api_key.user_id.clone() || 
                        group.admin_invites.iter().any(|invite_id| {
                            INVITES_BY_ID_HASHTABLE.with(|store| {
                                store.borrow()
                                    .get(invite_id)
                                    .map(|invite| invite.invitee_id == GroupInviteeID::User(requester_api_key.user_id.clone()))
                                    .unwrap_or(false)
                            })
                        });

        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::Create) {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();

        // Create new invite

        let invite_id = match create_req.id {
            Some(id) => GroupInviteID(id.to_string()),
            None => GroupInviteID(generate_uuidv4(IDPrefix::GroupInvite)),
        };

        let now = ic_cdk::api::time() / 1_000_000;

        // 4. Parse and validate grantee ID if provided (not required for deferred links)
        let (invitee_id, redeem_code) = if let Some(invitee_user_id) = create_req.invitee_id {
            // check if invitee_id === "PUBLIC"
            if invitee_user_id == "PUBLIC" {
                (GroupInviteeID::Public, Some("PUBLIC".to_string()))
            } else {
                (GroupInviteeID::User(UserID(invitee_user_id)), None)
            }
        } else {
            let _placeholder_id = PlaceholderGroupInviteeID(
                generate_uuidv4(IDPrefix::PlaceholderGroupInviteeID)
            );
            let _placeholder_invitee = GroupInviteeID::PlaceholderGroupInvitee(_placeholder_id.clone());
            mark_claimed_uuid(&_placeholder_id.clone().to_string());
            let redeem_code = format!("REDEEM_{}", ic_cdk::api::time());
            (_placeholder_invitee, Some(redeem_code))
        };


        let new_invite = GroupInvite {
            id: invite_id.clone(),
            group_id: group_id.clone(),
            inviter_id: requester_api_key.user_id.clone(),
            invitee_id,
            role: create_req.role.unwrap_or(GroupRole::Member),
            note: create_req.note.unwrap_or("".to_string()),
            created_at: now,
            last_modified_at: now, 
            active_from: create_req.active_from.unwrap_or(0),
            expires_at: create_req.expires_at.unwrap_or(-1),
            from_placeholder_invitee: None,
            labels: vec![],
            redeem_code,
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };
        update_external_id_mapping(None, new_invite.external_id.clone(), Some(invite_id.0.to_string()));

        // Update all relevant state stores
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(invite_id.clone(), new_invite.clone());
        });

        // Update group's invite lists
        GROUPS_BY_ID_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(group) = store.get_mut(&group_id) {
                match new_invite.role {
                    GroupRole::Admin => {
                        group.admin_invites.push(invite_id.clone());
                        group.member_invites.push(invite_id.clone());
                    },
                    GroupRole::Member => group.member_invites.push(invite_id.clone()),
                }
            }
        });

        // Update user's group invites
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            store.entry(new_invite.invitee_id.clone())
                .or_insert_with(Vec::new)
                .push(invite_id.clone());
        });

        mark_claimed_uuid(&invite_id.clone().to_string());

        // Fire webhook if we have active ones - create snapshot with group data
        if !active_webhooks.is_empty() {
            let after_snap = GroupInviteWebhookData {
                group: GROUPS_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&group_id).cloned()
                ),
                group_invite: INVITES_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&invite_id).cloned()
                ),
            };

            fire_group_invite_webhook(
                WebhookEventLabel::GroupInviteCreated,
                active_webhooks,
                Some(before_snap),
                Some(after_snap),
                Some("Invite created".to_string())
            );
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Group Invite {}", 
                requester_api_key.user_id,
                invite_id.0
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            CreateGroup_InviteResponse::ok(&new_invite.cast_fe(&requester_api_key.user_id)).encode()
        )

    }
    
    pub async fn update_group_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateGroupInviteRequestBody>(body).unwrap();
        
        if let Err(validation_err) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_err.field, validation_err.message)
                ).encode()
            );
        }

        let invite_id = update_req.id;

        // Get existing invite
        let mut invite = match INVITES_BY_ID_HASHTABLE.with(|store| 
            store.borrow().get(&invite_id).cloned()
        ) {
            Some(invite) => invite,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let active_webhooks = get_active_group_invite_webhooks(&invite.group_id, WebhookEventLabel::GroupInviteUpdated);
        let before_snap = GroupInviteWebhookData {
            group: GROUPS_BY_ID_HASHTABLE.with(|store| 
                store.borrow().get(&invite.group_id).cloned()
            ),
            group_invite: Some(invite.clone()),
        };
        
        // Check if user is authorized (owner or admin)
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        let is_authorized = is_owner || 
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .get(&invite_id)
                .map(|invite| invite.inviter_id == requester_api_key.user_id)
                .unwrap_or(false)
        });

        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::Edit) {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();
        

        // If role is being updated, we need to update the group's invite lists
        if let Some(new_role) = update_req.role {
            if new_role != invite.role {
                GROUPS_BY_ID_HASHTABLE.with(|store| {
                    let mut store = store.borrow_mut();
                    if let Some(group) = store.get_mut(&invite.group_id) {
                        // Remove from old role's list
                        match invite.role {
                            GroupRole::Admin => {
                                if let Some(pos) = group.admin_invites.iter().position(|id| *id == invite_id) {
                                    group.admin_invites.remove(pos);
                                }
                                if let Some(pos) = group.member_invites.iter().position(|id| *id == invite_id) {
                                    group.member_invites.remove(pos);
                                }
                            },
                            GroupRole::Member => {
                                if let Some(pos) = group.member_invites.iter().position(|id| *id == invite_id) {
                                    group.member_invites.remove(pos);
                                }
                            },
                        }
                        // Add to new role's list
                        match new_role {
                            GroupRole::Admin => {
                                if !group.admin_invites.contains(&invite_id) {
                                    group.admin_invites.push(invite_id.clone());
                                }
                                if !group.member_invites.contains(&invite_id) {
                                    group.member_invites.push(invite_id.clone());
                                }
                            },
                            GroupRole::Member => {
                                if !group.member_invites.contains(&invite_id) {
                                    group.member_invites.push(invite_id.clone());
                                }
                            },
                        }
                    }
                });
                invite.role = new_role;
            }
        }

        // Update other fields if provided
        if let Some(active_from) = update_req.active_from {
            invite.active_from = active_from;
        }

        if let Some(expires_at) = update_req.expires_at {
            invite.expires_at = expires_at;
        }
        if let Some(note) = update_req.note {
            invite.note = note;
        }

        invite.last_modified_at = ic_cdk::api::time();

        if let Some(external_id) = update_req.external_id.clone() {
            let old_external_id = invite.external_id.clone();
            let new_external_id = Some(ExternalID(external_id.clone()));
            invite.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(invite.id.to_string())
            );
        }
        if let Some(external_payload) = update_req.external_payload.clone() {
            invite.external_payload = Some(ExternalPayload(external_payload));
        }

        // Update state
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(invite.id.clone(), invite.clone());
        });

        // Fire webhook if we have active ones - create snapshot with group data
        if !active_webhooks.is_empty() {
            let after_snap = GroupInviteWebhookData {
                group: GROUPS_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&invite.group_id).cloned()
                ),
                group_invite: INVITES_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&invite_id).cloned()
                ),
            };
            fire_group_invite_webhook(
                WebhookEventLabel::GroupInviteUpdated,
                active_webhooks,
                Some(before_snap),
                Some(after_snap),
                Some("Invite updated".to_string())
            );
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Group Invite {}", 
                requester_api_key.user_id,
                invite_id.0
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            UpdateGroup_InviteResponse::ok(&invite.cast_fe(&requester_api_key.user_id)).encode()
        )
    }
    

    pub async fn delete_group_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let delete_req = match serde_json::from_slice::<DeleteGroup_InviteRequest>(request.body()) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_err) = delete_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_err.field, validation_err.message)
                ).encode()
            );
        }
    
        // Get invite to verify it exists
        let invite = match INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().get(&delete_req.id).cloned()) {
            Some(invite) => invite,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = invite.external_id.clone();
        let old_internal_id = Some(invite.id.clone().to_string());
    
        // Check if user is authorized (group owner, admin, or invite recipient)
        let is_authorized = GROUPS_BY_ID_HASHTABLE.with(|store| {
            if let Some(group) = store.borrow().get(&invite.group_id) {
                group.owner == requester_api_key.user_id || 
                group.admin_invites.contains(&invite.id)
            } else {
                false
            }
        });

        let prestate = snapshot_prestate();
        
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::Delete) {
            return create_auth_error_response();
        }
    
        // Remove from all state stores
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&delete_req.id);
        });
    
        // Update group's invite lists
        GROUPS_BY_ID_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(group) = store.get_mut(&invite.group_id) {
                match invite.role {
                    GroupRole::Admin => {
                        if let Some(pos) = group.admin_invites.iter().position(|id| *id == delete_req.id) {
                            group.admin_invites.remove(pos);
                        }
                    },
                    GroupRole::Member => {
                        if let Some(pos) = group.member_invites.iter().position(|id| *id == delete_req.id) {
                            group.member_invites.remove(pos);
                        }
                    },
                }
            }
        });
        
        // Update user's group invites
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(invites) = store.get_mut(&invite.invitee_id) {
                if let Some(pos) = invites.iter().position(|id| *id == delete_req.id) {
                    invites.remove(pos);
                }
            }
        });

        update_external_id_mapping(old_external_id, None, old_internal_id);

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Group Invite {}", 
                requester_api_key.user_id,
                delete_req.id.0
            ).to_string()
        ));
        
        create_response(
            StatusCode::OK,
            DeleteGroup_InviteResponse::ok(&DeletedGroup_InviteData {
                id: delete_req.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn redeem_group_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        
        // Parse request body
        let body: &[u8] = request.body();
        let redeem_request = match serde_json::from_slice::<RedeemGroupInviteRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // Validate request
        if redeem_request.validate_body().is_err() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            );
        }
    
        // Convert invite_id string to GroupInviteID
        let invite_id = GroupInviteID(redeem_request.invite_id);
    
        // Get existing invite
        let invite = match INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&invite_id).cloned()
        }) {
            Some(invite) => invite,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Invite not found".to_string()).encode()
            ),
        };

        // validate the redeem code matches
        if let Some(redeem_code) = &invite.redeem_code {
            if redeem_request.redeem_code != *redeem_code {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid redeem code".to_string()).encode()
                );
            }
        } else {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invite does not have a redeem code".to_string()).encode()
            );
        }
    
        let prestate = snapshot_prestate();
    
        // Parse and validate the user_id
        let new_user_id = UserID(redeem_request.user_id);
        let new_invitee = GroupInviteeID::User(new_user_id.clone());
    
        // Handle differently based on invitee_id type
        if invite.invitee_id == GroupInviteeID::Public {
            // For Public invites, create a new invite rather than modifying the original
            let new_invite_id = GroupInviteID(generate_uuidv4(IDPrefix::GroupInvite));
            let now = ic_cdk::api::time();
            
            // Create a new invite with duplicated fields but user-specific changes
            let new_invite = GroupInvite {
                id: new_invite_id.clone(),
                group_id: invite.group_id.clone(),
                inviter_id: invite.inviter_id.clone(),
                invitee_id: new_invitee,
                role: GroupRole::Member, // Default to Member role when redeeming
                note: invite.note.clone(),
                created_at: now,
                last_modified_at: now,
                active_from: invite.active_from,
                expires_at: invite.expires_at,
                redeem_code: None,
                from_placeholder_invitee: Some(invite.invitee_id.clone().to_string()),
                labels: invite.labels.clone(),
                external_id: invite.external_id.clone(),
                external_payload: invite.external_payload.clone(),
            };
    
            // Add the new invite to the invites store
            INVITES_BY_ID_HASHTABLE.with(|store| {
                store.borrow_mut().insert(new_invite_id.clone(), new_invite.clone());
            });
    
            // Update user's group invites list with the new invite
            USERS_INVITES_LIST_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                store.entry(new_invite.invitee_id.clone())
                    .or_insert_with(Vec::new)
                    .push(new_invite_id.clone());
            });
    
            // Update group's member invites
            GROUPS_BY_ID_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                if let Some(group) = store.get_mut(&invite.group_id) {
                    group.member_invites.push(new_invite_id.clone());
                }
            });
    
            mark_claimed_uuid(&new_invite_id.0);
    
            snapshot_poststate(prestate, Some(
                format!(
                    "{}: Redeem Public Group Invite {} as {}",
                    requester_api_key.user_id,
                    invite_id.clone(),
                    new_invite_id.clone()
                ).to_string()
            ));
    
            create_response(
                StatusCode::OK,
                serde_json::to_vec(&RedeemGroupInviteResponseData {
                    invite: new_invite.cast_fe(&requester_api_key.user_id),
                }).expect("Failed to serialize response")
            )
        } else if invite.invitee_id.to_string().starts_with(IDPrefix::PlaceholderGroupInviteeID.as_str()) {
            // Handle original placeholder invitee case
            if invite.from_placeholder_invitee.is_some() {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invite has already been redeemed".to_string()).encode()
                );
            }
    
            // Update existing invite for placeholder invitees
            let mut updated_invite = invite.clone();
            updated_invite.from_placeholder_invitee = Some(invite.invitee_id.clone().to_string());
            updated_invite.invitee_id = new_invitee;
            updated_invite.role = GroupRole::Member; // Default to Member role when redeeming
            updated_invite.last_modified_at = ic_cdk::api::time();
            updated_invite.redeem_code = None;
    
            // Update state
            INVITES_BY_ID_HASHTABLE.with(|store| {
                store.borrow_mut().insert(invite_id.clone(), updated_invite.clone());
            });
    
            // Update user's group invites list
            USERS_INVITES_LIST_HASHTABLE.with(|store| {
                let mut store = store.borrow_mut();
                store.entry(updated_invite.invitee_id.clone())
                    .or_insert_with(Vec::new)
                    .push(invite_id.clone());
            });
    
            snapshot_poststate(prestate, Some(
                format!(
                    "{}: Redeem Group Invite {}",
                    requester_api_key.user_id,
                    invite_id.clone()
                ).to_string()
            ));
    
            create_response(
                StatusCode::OK,
                serde_json::to_vec(&RedeemGroupInviteResponseData {
                    invite: updated_invite.cast_fe(&requester_api_key.user_id),
                }).expect("Failed to serialize response")
            )
        } else {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invite is not a public or placeholder invite".to_string()).encode()
            );
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