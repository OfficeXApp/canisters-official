// src/rest/team_invites/handler.rs


pub mod team_invites_handlers {
    use crate::{
        core::{api::{permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}, webhooks::team_invites::{fire_team_invite_webhook, get_active_team_invite_webhooks}}, state::{drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemResourceID, SystemTableEnum}, team_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::{PlaceholderTeamInviteeID, TeamInviteID, TeamInviteeID, TeamRole}}, teams::{state::state::TEAMS_BY_ID_HASHTABLE, types::TeamID}, webhooks::types::WebhookEventLabel}, types::{IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, team_invites::types::{ CreateTeamInviteRequestBody, CreateTeam_InviteResponse, DeleteTeam_InviteRequest, DeleteTeam_InviteResponse, DeletedTeam_InviteData, ErrorResponse, GetTeam_InviteResponse, ListTeamInvitesRequestBody, ListTeamInvitesResponseData, ListTeam_InvitesResponse, RedeemTeamInviteRequest, RedeemTeamInviteResponseData, UpdateTeamInviteRequestBody, UpdateTeam_InviteRequest, UpdateTeam_InviteResponse}, teams::types::{ListTeamsRequestBody, ListTeamsResponseData}, webhooks::types::TeamInviteWebhookData}
        
    };
    use crate::core::state::team_invites::{
        types::TeamInvite,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub async fn get_team_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        let invite_id = TeamInviteID(params.get("invite_id").unwrap().to_string());
        
        let invite = INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&invite_id).cloned()
        });
    
        match invite {
            Some(invite) => {
                // Check if user is authorized (team owner, admin, or invitee)
                let is_authorized = TEAMS_BY_ID_HASHTABLE.with(|store| {
                    if let Some(team) = store.borrow().get(&invite.team_id) {
                        team.owner == requester_api_key.user_id.clone() || 
                        team.admin_invites.contains(&invite.id) ||
                        invite.invitee_id == TeamInviteeID::User(requester_api_key.user_id.clone())
                    } else {
                        false
                    }
                });

                let table_permissions = check_system_permissions(
                    SystemResourceID::Table(SystemTableEnum::Teams),
                    PermissionGranteeID::User(requester_api_key.user_id.clone())
                );
    
                if !is_authorized && !table_permissions.contains(&SystemPermissionType::View) {
                    return create_auth_error_response();
                }
    
                create_response(
                    StatusCode::OK,
                    GetTeam_InviteResponse::ok(&invite.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }
    
    pub async fn list_team_invites_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let query: ListTeamInvitesRequestBody = match serde_json::from_slice(request.body()) {
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
    
        let team_id = TeamID(query.team_id.clone());

        // Check if the team exists first
        let team_exists = TEAMS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().contains_key(&team_id)
        });
        if !team_exists {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(400, "Team not found".to_string()).encode()
            );
        }
    
        // Check if user is owner or admin
        let is_authorized = TEAMS_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .get(&team_id)
                .map(|team| {
                    team.owner == requester_api_key.user_id.clone() || 
                    team.admin_invites.iter().any(|invite_id| {
                        INVITES_BY_ID_HASHTABLE.with(|invite_store| {
                            invite_store.borrow()
                                .get(invite_id)
                                .map(|invite| invite.invitee_id == TeamInviteeID::User(requester_api_key.user_id.clone()))
                                .unwrap_or(false)
                        })
                    })
                })
                .unwrap_or(false)
        });
    
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Teams),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::View) {
            return create_auth_error_response();
        }
    
        let all_invites = TEAMS_BY_ID_HASHTABLE.with(|teams_store| {
            let teams = teams_store.borrow();
            teams.get(&team_id)
                .map(|team| INVITES_BY_ID_HASHTABLE.with(|invite_store| {
                    let invites = invite_store.borrow();
                    team.member_invites.iter()
                        .filter_map(|id| invites.get(id))
                        .cloned()
                        .collect::<Vec<_>>()
                }))
                .unwrap_or_default()
        });
    
        let start = if let Some(cursor) = query.cursor_down {
            match all_invites.iter().position(|i| i.id.0 == cursor) {
                Some(pos) => pos + 1,
                None => 0,
            }
        } else {
            0
        };
    
        let items = all_invites
            .iter()
            .skip(start)
            .take(query.page_size)
            .cloned()
            .collect::<Vec<_>>();
    
        let response_data = ListTeamInvitesResponseData {
            items: items.clone().into_iter().map(|invite| invite.cast_fe(&requester_api_key.user_id)).collect(),
            page_size: query.page_size,
            total: all_invites.len(),
            cursor_up: items.first().map(|i| i.id.0.clone()),
            cursor_down: items.last().map(|i| i.id.0.clone()),
        };
    
        create_response(
            StatusCode::OK,
            ListTeam_InvitesResponse::ok(&response_data).encode()
        )
    }
    
    pub async fn create_team_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateTeamInviteRequestBody>(body).unwrap();
        
        if let Err(validation_err) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_err.field, validation_err.message)
                ).encode()
            );
        }

        let team_id = TeamID(create_req.team_id);
        let active_webhooks = get_active_team_invite_webhooks(&team_id, WebhookEventLabel::TeamInviteCreated);

        let before_snap = TeamInviteWebhookData {
            team: TEAMS_BY_ID_HASHTABLE.with(|store| 
                store.borrow().get(&team_id).cloned()
            ),
            team_invite: None,
        };

        // Verify team exists and user has permission
        let team = match TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&team_id).cloned()) {
            Some(team) => team,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Check if user is authorized (owner or admin)
        let is_authorized = team.owner == requester_api_key.user_id.clone() || 
                        team.admin_invites.iter().any(|invite_id| {
                            INVITES_BY_ID_HASHTABLE.with(|store| {
                                store.borrow()
                                    .get(invite_id)
                                    .map(|invite| invite.invitee_id == TeamInviteeID::User(requester_api_key.user_id.clone()))
                                    .unwrap_or(false)
                            })
                        });

        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Teams),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::Create) {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();

        // Create new invite

        let invite_id = match create_req.id {
            Some(id) => TeamInviteID(id.to_string()),
            None => TeamInviteID(generate_uuidv4(IDPrefix::TeamInvite)),
        };

        let now = ic_cdk::api::time();

        // 4. Parse and validate grantee ID if provided (not required for deferred links)
        let invitee_id = if let Some(invitee_user_id) = create_req.invitee_id {
            TeamInviteeID::User(UserID(invitee_user_id))
        } else {
            let _placeholder_id = PlaceholderTeamInviteeID(
                generate_uuidv4(IDPrefix::PlaceholderTeamInviteeID)
            );
            let _placeholder_invitee = TeamInviteeID::PlaceholderTeamInvitee(_placeholder_id.clone());
            mark_claimed_uuid(&_placeholder_id.clone().to_string());
            _placeholder_invitee
        };


        let new_invite = TeamInvite {
            id: invite_id.clone(),
            team_id: team_id.clone(),
            inviter_id: requester_api_key.user_id.clone(),
            invitee_id,
            role: create_req.role,
            note: create_req.note.unwrap_or("".to_string()),
            created_at: now,
            last_modified_at: now,
            active_from: create_req.active_from.unwrap_or(0),
            expires_at: create_req.expires_at.unwrap_or(-1),
            from_placeholder_invitee: None,
            tags: vec![],
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };
        update_external_id_mapping(None, new_invite.external_id.clone(), Some(invite_id.0.to_string()));

        // Update all relevant state stores
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(invite_id.clone(), new_invite.clone());
        });

        // Update team's invite lists
        TEAMS_BY_ID_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(team) = store.get_mut(&team_id) {
                match new_invite.role {
                    TeamRole::Admin => {
                        team.admin_invites.push(invite_id.clone());
                        team.member_invites.push(invite_id.clone());
                    },
                    TeamRole::Member => team.member_invites.push(invite_id.clone()),
                }
            }
        });

        // Update user's team invites
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            store.entry(new_invite.invitee_id.clone())
                .or_insert_with(Vec::new)
                .push(invite_id.clone());
        });

        mark_claimed_uuid(&invite_id.clone().to_string());

        // Fire webhook if we have active ones - create snapshot with team data
        if !active_webhooks.is_empty() {
            let after_snap = TeamInviteWebhookData {
                team: TEAMS_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&team_id).cloned()
                ),
                team_invite: INVITES_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&invite_id).cloned()
                ),
            };

            fire_team_invite_webhook(
                WebhookEventLabel::TeamInviteCreated,
                active_webhooks,
                Some(before_snap),
                Some(after_snap),
                Some("Invite created".to_string())
            );
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Team Invite {}", 
                requester_api_key.user_id,
                invite_id.0
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            CreateTeam_InviteResponse::ok(&new_invite.cast_fe(&requester_api_key.user_id)).encode()
        )

    }
    
    pub async fn update_team_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateTeamInviteRequestBody>(body).unwrap();
        
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
        let active_webhooks = get_active_team_invite_webhooks(&invite.team_id, WebhookEventLabel::TeamInviteUpdated);
        let before_snap = TeamInviteWebhookData {
            team: TEAMS_BY_ID_HASHTABLE.with(|store| 
                store.borrow().get(&invite.team_id).cloned()
            ),
            team_invite: Some(invite.clone()),
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
            SystemResourceID::Table(SystemTableEnum::Teams),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::Edit) {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();
        

        // If role is being updated, we need to update the team's invite lists
        if let Some(new_role) = update_req.role {
            if new_role != invite.role {
                TEAMS_BY_ID_HASHTABLE.with(|store| {
                    let mut store = store.borrow_mut();
                    if let Some(team) = store.get_mut(&invite.team_id) {
                        // Remove from old role's list
                        match invite.role {
                            TeamRole::Admin => {
                                if let Some(pos) = team.admin_invites.iter().position(|id| *id == invite_id) {
                                    team.admin_invites.remove(pos);
                                }
                                if let Some(pos) = team.member_invites.iter().position(|id| *id == invite_id) {
                                    team.member_invites.remove(pos);
                                }
                            },
                            TeamRole::Member => {
                                if let Some(pos) = team.member_invites.iter().position(|id| *id == invite_id) {
                                    team.member_invites.remove(pos);
                                }
                            },
                        }
                        // Add to new role's list
                        match new_role {
                            TeamRole::Admin => {
                                if !team.admin_invites.contains(&invite_id) {
                                    team.admin_invites.push(invite_id.clone());
                                }
                                if !team.member_invites.contains(&invite_id) {
                                    team.member_invites.push(invite_id.clone());
                                }
                            },
                            TeamRole::Member => {
                                if !team.member_invites.contains(&invite_id) {
                                    team.member_invites.push(invite_id.clone());
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

        // Fire webhook if we have active ones - create snapshot with team data
        if !active_webhooks.is_empty() {
            let after_snap = TeamInviteWebhookData {
                team: TEAMS_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&invite.team_id).cloned()
                ),
                team_invite: INVITES_BY_ID_HASHTABLE.with(|store| 
                    store.borrow().get(&invite_id).cloned()
                ),
            };
            fire_team_invite_webhook(
                WebhookEventLabel::TeamInviteUpdated,
                active_webhooks,
                Some(before_snap),
                Some(after_snap),
                Some("Invite updated".to_string())
            );
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Team Invite {}", 
                requester_api_key.user_id,
                invite_id.0
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            UpdateTeam_InviteResponse::ok(&invite.cast_fe(&requester_api_key.user_id)).encode()
        )
    }
    

    pub async fn delete_team_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let delete_req = match serde_json::from_slice::<DeleteTeam_InviteRequest>(request.body()) {
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
    
        // Check if user is authorized (team owner, admin, or invite recipient)
        let is_authorized = TEAMS_BY_ID_HASHTABLE.with(|store| {
            if let Some(team) = store.borrow().get(&invite.team_id) {
                team.owner == requester_api_key.user_id || 
                team.admin_invites.contains(&invite.id)
            } else {
                false
            }
        });

        let prestate = snapshot_prestate();
        
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Teams),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_authorized && !table_permissions.contains(&SystemPermissionType::Delete) {
            return create_auth_error_response();
        }
    
        // Remove from all state stores
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&delete_req.id);
        });
    
        // Update team's invite lists
        TEAMS_BY_ID_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(team) = store.get_mut(&invite.team_id) {
                match invite.role {
                    TeamRole::Admin => {
                        if let Some(pos) = team.admin_invites.iter().position(|id| *id == delete_req.id) {
                            team.admin_invites.remove(pos);
                        }
                    },
                    TeamRole::Member => {
                        if let Some(pos) = team.member_invites.iter().position(|id| *id == delete_req.id) {
                            team.member_invites.remove(pos);
                        }
                    },
                }
            }
        });
        
        // Update user's team invites
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
                "{}: Delete Team Invite {}", 
                requester_api_key.user_id,
                delete_req.id.0
            ).to_string()
        ));
        
        create_response(
            StatusCode::OK,
            DeleteTeam_InviteResponse::ok(&DeletedTeam_InviteData {
                id: delete_req.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn redeem_team_invite_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        
        // Parse request body
        let body: &[u8] = request.body();
        let redeem_request = match serde_json::from_slice::<RedeemTeamInviteRequest>(body) {
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
    
        // Convert invite_id string to TeamInviteID
        let invite_id = TeamInviteID(redeem_request.invite_id);
    
        // Get existing invite
        let mut invite = match INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&invite_id).cloned()
        }) {
            Some(invite) => invite,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Invite not found".to_string()).encode()
            ),
        };
    
        // Check if inviter is actually a placeholder and not already redeemed
        if !invite.invitee_id.to_string().starts_with(IDPrefix::PlaceholderTeamInviteeID.as_str()) {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invite is not a placeholder invite".to_string()).encode()
            );
        }
    
        if invite.from_placeholder_invitee.is_some() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invite has already been redeemed".to_string()).encode()
            );
        }

        let prestate = snapshot_prestate();
    
        // Parse and validate the user_id
        let new_invitee = TeamInviteeID::User(UserID(redeem_request.user_id));
    
        // Store placeholder ID and update invite
        invite.from_placeholder_invitee = Some(PlaceholderTeamInviteeID(invite.invitee_id.to_string().clone()));
        invite.invitee_id = new_invitee;
        invite.role = TeamRole::Member; // Default to Member role when redeeming
        invite.last_modified_at = ic_cdk::api::time() / 1_000_000;
    
        // Update state
        INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(invite_id.clone(), invite.clone());
        });
    
        // Update user's team invites list
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            store.entry(invite.invitee_id.clone())
                .or_insert_with(Vec::new)
                .push(invite_id.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Redeem Team Invite {}", 
                requester_api_key.user_id,
                invite_id.clone()
            ).to_string()
        ));
    
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&RedeemTeamInviteResponseData {
                invite: invite.cast_fe(&requester_api_key.user_id),
            }).expect("Failed to serialize response")
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