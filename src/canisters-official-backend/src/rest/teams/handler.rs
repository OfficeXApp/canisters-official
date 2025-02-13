// src/rest/teams/handler.rs


pub mod teams_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{drives::{state::state::{DRIVE_ID, OWNER_ID}, types::DriveID}, team_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::Team_Invite}, teams::{state::state::{TEAMS_BY_ID_HASHTABLE, TEAMS_BY_TIME_LIST}, types::{Team, TeamID}}}, types::{IDPrefix, PublicKeyICP}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, teams::types::{CreateTeamResponse, DeleteTeamRequestBody, DeleteTeamResponse, DeletedTeamData, ErrorResponse, GetTeamResponse, ListTeamsResponseData, TeamResponse, UpdateTeamResponse, UpsertTeamRequestBody}}
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_team_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let id = TeamID(params.get("team_id").unwrap().to_string());

        // Only owner can read teams for now
        let is_authorized = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);

        if !is_authorized {
            return create_auth_error_response();
        }

        let team = TEAMS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&id).cloned()
        });

        match team {
            Some(team) => create_response(
                StatusCode::OK,
                GetTeamResponse::ok(&team).encode()
            ),
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }


    pub fn list_teams_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can list teams for now
        let is_authorized = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);

        if !is_authorized {
            return create_auth_error_response();
        }

        let teams = TEAMS_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .values()
                .cloned()
                .collect::<Vec<_>>()
        });

        let response_data = ListTeamsResponseData {
            items: teams.clone(),
            page_size: 50, // Using the default page size
            total: teams.len(),
            cursor_up: None,
            cursor_down: None,
        };
    
        // Wrap it in a TeamResponse and encode
        create_response(
            StatusCode::OK,
            TeamResponse::ok(&response_data).encode()
        )

    }

    pub fn upsert_team_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can create/update teams for now
        let is_authorized = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);

        if !is_authorized {
            return create_auth_error_response();
        }

        // Parse request body
        let body: &[u8] = req.body();
        
        if let Ok(req) = serde_json::from_slice::<UpsertTeamRequestBody>(body) {
            match req {
                UpsertTeamRequestBody::Create(create_req) => {
                    let drive_id_suffix = format!("__DriveID_{}", ic_cdk::api::id().to_text());
                    let team_id = TeamID(generate_unique_id(IDPrefix::Team, &drive_id_suffix));
                    let now = ic_cdk::api::time();

                    // Create new team
                    let new_team = Team {
                        id: team_id.clone(),
                        name: create_req.name,
                        owner: requester_api_key.user_id.clone(),
                        private_note: if is_authorized { create_req.private_note } else { None },
                        public_note: create_req.public_note,
                        admin_invites: Vec::new(),
                        member_invites: Vec::new(),
                        created_at: now,
                        last_modified_at: now,
                        drive_id: DRIVE_ID.with(|id| id.clone()),
                    };

                    // Update state
                    TEAMS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(team_id.clone(), new_team.clone());
                    });

                    TEAMS_BY_TIME_LIST.with(|list| {
                        list.borrow_mut().push(team_id.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        CreateTeamResponse::ok(&new_team).encode()
                    )
                },
                UpsertTeamRequestBody::Update(update_req) => {
                    let team_id = TeamID(update_req.id);
                    
                    // Get existing team
                    let mut team = match TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&team_id).cloned()) {
                        Some(team) => team,
                        None => return create_response(
                            StatusCode::NOT_FOUND,
                            ErrorResponse::not_found().encode()
                        ),
                    };

                    // Update fields
                    if let Some(name) = update_req.name {
                        team.name = name;
                    }
                    if let Some(public_note) = update_req.public_note {
                        team.public_note = Some(public_note);
                    }
                    if let Some(private_note) = update_req.private_note {
                        if (is_authorized) {
                            team.private_note = Some(private_note);
                        }
                    }
                    team.last_modified_at = ic_cdk::api::time();

                    // Update state
                    TEAMS_BY_ID_HASHTABLE.with(|store| {
                        store.borrow_mut().insert(team.id.clone(), team.clone());
                    });

                    create_response(
                        StatusCode::OK,
                        UpdateTeamResponse::ok(&team).encode()
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

    pub fn delete_team_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Only owner can delete teams for now
        let is_authorized = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
    
        if !is_authorized {
            return create_auth_error_response();
        }
    
        // Parse request body
        let body: &[u8] = req.body();
        let delete_request = match serde_json::from_slice::<DeleteTeamRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        let team_id = TeamID(delete_request.id.clone());
    
        // Get team to verify it exists
        let team = match TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&team_id).cloned()) {
            Some(team) => team,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
    
    
        // First, get all invites to know which users we need to update
        let invites_to_remove = INVITES_BY_ID_HASHTABLE.with(|store| {
            let store = store.borrow();
            team.member_invites.clone().iter()
                .filter_map(|invite_id| store.get(invite_id).cloned())
                .collect::<Vec<Team_Invite>>()
        });
    
        // Remove invites from INVITES_BY_ID_HASHTABLE
        INVITES_BY_ID_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            for invite_id in &team.member_invites {
                store.remove(invite_id);
            }
        });
    
        // Remove invites from USERS_INVITES_LIST_HASHTABLE
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            // For each invite we're removing, update the corresponding user's invite list
            for invite in &invites_to_remove {
                if let Some(user_invites) = store.get_mut(&invite.invitee_id) {
                    user_invites.retain(|id| !team.member_invites.contains(id));
                }
            }
        });
    
        // Remove team from TEAMS_BY_ID_HASHTABLE
        TEAMS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&team_id);
        });
    
        // Remove team from TEAMS_BY_TIME_LIST
        TEAMS_BY_TIME_LIST.with(|list| {
            let mut list = list.borrow_mut();
            if let Some(pos) = list.iter().position(|id| *id == team_id) {
                list.remove(pos);
            }
        });
    
        create_response(
            StatusCode::OK,
            DeleteTeamResponse::ok(&DeletedTeamData {
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