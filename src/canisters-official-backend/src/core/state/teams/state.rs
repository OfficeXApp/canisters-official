// src/core/state/teams/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::{state::{team_invites::types::TeamInviteID, teams::types::{Team, TeamID}}, types::UserID};
    
    thread_local! {
        // default is to use the api key id to lookup the api key
        pub(crate) static TEAMS_BY_ID_HASHTABLE: RefCell<HashMap<TeamID, Team>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub(crate) static TEAMS_BY_TIME_LIST: RefCell<Vec<TeamID>> = RefCell::new(Vec::new());
        // track in hashtable users list of teams
        pub(crate) static USERS_TEAMS_HASHTABLE: RefCell<HashMap<UserID, Vec<TeamInviteID>>> = RefCell::new(HashMap::new());
    }

}


