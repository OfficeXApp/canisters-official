// src/core/state/team_invites/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::{state::team_invites::types::{TeamInviteID, TeamInviteeID, Team_Invite}, types::UserID};
    
    thread_local! {
        pub(crate) static INVITES_BY_ID_HASHTABLE: RefCell<HashMap<TeamInviteID, Team_Invite>> = RefCell::new(HashMap::new());
        pub(crate) static USERS_INVITES_LIST_HASHTABLE: RefCell<HashMap<TeamInviteeID, Vec<TeamInviteID>>> = RefCell::new(HashMap::new());
    }

}


