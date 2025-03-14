// src/core/state/group_invites/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::{state::group_invites::types::{GroupInviteID, GroupInviteeID, GroupInvite}, types::UserID};
    
    thread_local! {
        pub(crate) static INVITES_BY_ID_HASHTABLE: RefCell<HashMap<GroupInviteID, GroupInvite>> = RefCell::new(HashMap::new());
        pub(crate) static USERS_INVITES_LIST_HASHTABLE: RefCell<HashMap<GroupInviteeID, Vec<GroupInviteID>>> = RefCell::new(HashMap::new());
    }

}


