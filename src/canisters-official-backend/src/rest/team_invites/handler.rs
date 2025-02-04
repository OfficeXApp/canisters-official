// src/rest/team_invites/handler.rs


pub mod team_invites_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{drives::state::state::OWNER_ID, team_invites::{state::state::TEAM_INVITES_BY_ID_HASHTABLE, types::{TeamInviteID, TeamRole}}, teams::{state::state::{TEAMS_BY_ID_HASHTABLE, USERS_TEAMS_HASHTABLE}, types::TeamID}}, types::{PublicKeyBLS, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, team_invites::types::{ CreateTeam_InviteResponse, DeleteTeam_InviteRequest, DeleteTeam_InviteResponse, DeletedTeam_InviteData, ErrorResponse, GetTeam_InviteResponse, ListTeamInvitesRequestBody, ListTeamInvitesResponseData, ListTeam_InvitesResponse, UpdateTeam_InviteRequest, UpdateTeam_InviteResponse, UpsertTeamInviteRequestBody}, teams::types::{ListTeamsRequestBody, ListTeamsResponseData}}
        
    };
    use crate::core::state::team_invites::{
        types::Team_Invite,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub fn get_team_invite_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let invite_id = TeamInviteID(params.get("invite_id").unwrap().to_string());
        
        let invite = TEAM_INVITES_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&invite_id).cloned()
        });
    
        match invite {
            Some(invite) => {
                // Check if user is authorized (team owner, admin, or invitee)
                let is_authorized = TEAMS_BY_ID_HASHTABLE.with(|store| {
                    if let Some(team) = store.borrow().get(&invite.team_id) {
                        team.owner == requester_api_key.user_id || 
                        team.admin_invites.contains(&invite.id) ||
                        invite.invitee_id == requester_api_key.user_id
                    } else {
                        false
                    }
                });
    
                if !is_authorized {
                    return create_auth_error_response();
                }
    
                create_response(
                    StatusCode::OK,
                    GetTeam_InviteResponse::ok(&invite).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }
    
    pub fn list_team_invites_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
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
                    team.owner == requester_api_key.user_id || 
                    team.admin_invites.iter().any(|invite_id| {
                        TEAM_INVITES_BY_ID_HASHTABLE.with(|invite_store| {
                            invite_store.borrow()
                                .get(invite_id)
                                .map(|invite| invite.invitee_id == requester_api_key.user_id)
                                .unwrap_or(false)
                        })
                    })
                })
                .unwrap_or(false)
        });
    
        if !is_authorized {
            return create_auth_error_response();
        }
    
        let all_invites = TEAMS_BY_ID_HASHTABLE.with(|teams_store| {
            let teams = teams_store.borrow();
            teams.get(&team_id)
                .map(|team| TEAM_INVITES_BY_ID_HASHTABLE.with(|invite_store| {
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
            items: items.clone(),
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
    
    pub fn upsert_team_invite_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let body: &[u8] = req.body();
        
        if let Ok(req) = serde_json::from_slice::<UpsertTeamInviteRequestBody>(body) {
            match req {
                UpsertTeamInviteRequestBody::Create(create_req) => {
                    let team_id = TeamID(create_req.team_id);

                    // Verify team exists and user has permission
                    let team = match TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&team_id).cloned()) {
                        Some(team) => team,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };

                    // Check if user is authorized (owner or admin)
                    let is_authorized = team.owner == requester_api_key.user_id || 
                                    team.admin_invites.iter().any(|invite_id| {
                                        TEAM_INVITES_BY_ID_HASHTABLE.with(|store| {
                                            store.borrow()
                                                .get(invite_id)
                                                .map(|invite| invite.invitee_id == requester_api_key.user_id)
                                                .unwrap_or(false)
                                        })
                                    });

                    if !is_authorized {
                        return create_auth_error_response();
                    }

                    // Create new invite
                    let invite_id = TeamInviteID(generate_unique_id("InviteID", ""));
                    let now = ic_cdk::api::time();

                    let new_invite = Team_Invite {
                        id: invite_id.clone(),
                        team_id: team_id.clone(),
                        inviter_id: requester_api_key.user_id.clone(),
                        invitee_id: UserID(create_req.invitee_id),
                        role: create_req.role,
                        created_at: now,
                        last_modified_at: now,
                        active_from: create_req.active_from.unwrap_or(0),
                        expires_at: create_req.expires_at.unwrap_or(-1),
                    };

                    // Update all relevant state stores
                    TEAM_INVITES_BY_ID_HASHTABLE.with(|store| {
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
                    USERS_TEAMS_HASHTABLE.with(|store| {
                        let mut store = store.borrow_mut();
                        store.entry(new_invite.invitee_id.clone())
                            .or_insert_with(Vec::new)
                            .push(invite_id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        CreateTeam_InviteResponse::ok(&new_invite).encode()
                    )
                },
                UpsertTeamInviteRequestBody::Update(update_req) => {
                    let invite_id = update_req.id;

                    // Get existing invite
                    let mut invite = match TEAM_INVITES_BY_ID_HASHTABLE.with(|store| 
                        store.borrow().get(&invite_id).cloned()
                    ) {
                        Some(invite) => invite,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };
                    
                    // Check if user is authorized (owner or admin)
                    let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
                    let is_authorized = is_owner || 
                    TEAM_INVITES_BY_ID_HASHTABLE.with(|store| {
                        store.borrow()
                            .get(&invite_id)
                            .map(|invite| invite.inviter_id == requester_api_key.user_id)
                            .unwrap_or(false)
                    });

                    if !is_authorized {
                        return create_auth_error_response();
                    }

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

                    invite.last_modified_at = ic_cdk::api::time();

                    // Update state
                    TEAM_INVITES_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(invite.id.clone(), invite.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateTeam_InviteResponse::ok(&invite).encode()
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
    
    pub fn delete_team_invite_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Parse request body
        let delete_req = match serde_json::from_slice::<DeleteTeam_InviteRequest>(req.body()) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // Get invite to verify it exists
        let invite = match TEAM_INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().get(&delete_req.id).cloned()) {
            Some(invite) => invite,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
    
        // Check if user is authorized (team owner, admin, or invite recipient)
        let is_authorized = TEAMS_BY_ID_HASHTABLE.with(|store| {
            if let Some(team) = store.borrow().get(&invite.team_id) {
                team.owner == requester_api_key.user_id || 
                team.admin_invites.contains(&invite.id)
            } else {
                false
            }
        });
    
        if !is_authorized {
            return create_auth_error_response();
        }
    
        // Remove from all state stores
        TEAM_INVITES_BY_ID_HASHTABLE.with(|store| {
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
        USERS_TEAMS_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            if let Some(invites) = store.get_mut(&invite.invitee_id) {
                if let Some(pos) = invites.iter().position(|id| *id == delete_req.id) {
                    invites.remove(pos);
                }
            }
        });
        
        create_response(
            StatusCode::OK,
            DeleteTeam_InviteResponse::ok(&DeletedTeam_InviteData {
                id: delete_req.id,
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