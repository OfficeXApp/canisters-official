// src/core/state/team_invites/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::team_invites::types::{TeamInviteID, Team_Invite};
    
    thread_local! {
        pub static TEAM_INVITES_BY_ID_HASHTABLE: RefCell<HashMap<TeamInviteID, Team_Invite>> = RefCell::new(HashMap::new());
    }

}


