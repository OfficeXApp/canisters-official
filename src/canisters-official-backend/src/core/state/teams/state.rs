// src/core/state/teams/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;
    use crate::{debug_log, rest::teams::types::{ValidateTeamResponseData}};
    use serde_json::json;

    use crate::core::{state::{drives::state::state::URL_ENDPOINT, team_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::{TeamInviteID, TeamInviteeID}}, teams::types::{Team, TeamID}}, types::UserID};
    
    thread_local! {
        // default is to use the api key id to lookup the api key
        pub(crate) static TEAMS_BY_ID_HASHTABLE: RefCell<HashMap<TeamID, Team>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub(crate) static TEAMS_BY_TIME_LIST: RefCell<Vec<TeamID>> = RefCell::new(Vec::new());
    }

    pub fn is_team_admin(user_id: &UserID, team_id: &TeamID) -> bool {
        TEAMS_BY_ID_HASHTABLE.with(|teams| {
            if let Some(team) = teams.borrow().get(team_id) {
                // Check if user is the owner
                if team.owner == *user_id {
                    return true;
                }

                // Check admin invites
                for invite_id in &team.admin_invites {
                    if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).cloned()) {
                        if invite.invitee_id == TeamInviteeID::User(user_id.clone()) {
                            let current_time = ic_cdk::api::time();
                            if invite.active_from <= current_time && 
                               (invite.expires_at <= 0 || invite.expires_at > current_time as i64) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        })
    }

    pub fn is_user_on_local_team(user_id: &UserID, team: &Team) -> bool {
        // Check if user is the owner
        if team.owner == *user_id {
            return true;
        }
    
        // Check member invites (which includes admin invites)
        for invite_id in &team.member_invites {
            if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).cloned()) {
                if invite.invitee_id == TeamInviteeID::User(user_id.clone()) {
                    let current_time = ic_cdk::api::time();
                    if invite.active_from <= current_time && 
                       (invite.expires_at <= 0 || invite.expires_at > current_time as i64) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub async fn is_user_on_team(user_id: &UserID, team_id: &TeamID) -> bool {
        let team_opt = TEAMS_BY_ID_HASHTABLE.with(|teams| teams.borrow().get(team_id).cloned());
        
        if let Some(team) = team_opt {
            // If it's our own drive's team, use local validation
            if team.url_endpoint == URL_ENDPOINT.with(|url| url.borrow().clone()) {
                return is_user_on_local_team(user_id, &team);
            }
    
            // It's an external team, make HTTP call to their validate endpoint
            let validation_url = format!("{}/teams/validate", team.url_endpoint.0.trim_end_matches('/'));
            
            let validation_body = json!({
                "team_id": team_id.0,
                "user_id": user_id.0,
            });
    
            let request = CanisterHttpRequestArgument {
                url: validation_url,
                method: HttpMethod::POST,
                headers: vec![
                    HttpHeader {
                        name: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    },
                ],
                body: Some(serde_json::to_vec(&validation_body).unwrap_or_default()),
                max_response_bytes: Some(2048),
                transform: None,
            };
    
            match http_request(request, 100_000_000_000).await {
                Ok((response,)) => {
                    if response.status.0 != BigUint::from_u16(200).unwrap_or_default() {
                        debug_log!("External team validation failed with status: {}", response.status.0);
                        return false;
                    }
    
                    match serde_json::from_slice::<ValidateTeamResponseData>(&response.body) {
                        Ok(result) => result.is_member,
                        Err(e) => {
                            debug_log!("Failed to parse team validation response: {}", e);
                            false
                        }
                    }
                },
                Err((code, msg)) => {
                    debug_log!("External team validation request failed: {:?} - {}", code, msg);
                    false
                }
            }
        } else {
            false
        }
    }
}


