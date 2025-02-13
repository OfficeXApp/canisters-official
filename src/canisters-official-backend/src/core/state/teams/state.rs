// src/core/state/teams/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::{state::{team_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::TeamInviteID}, teams::types::{Team, TeamID}}, types::UserID};
    
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
                        if invite.invitee_id == *user_id {
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

    pub fn is_user_on_team(user_id: &UserID, team_id: &TeamID) -> bool {
        TEAMS_BY_ID_HASHTABLE.with(|teams| {
            if let Some(team) = teams.borrow().get(team_id) {
                // Check if user is the owner
                if team.owner == *user_id {
                    return true;
                }

                // Check member invites (which includes admin invites)
                for invite_id in &team.member_invites {
                    if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).cloned()) {
                        if invite.invitee_id == *user_id {
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
}


