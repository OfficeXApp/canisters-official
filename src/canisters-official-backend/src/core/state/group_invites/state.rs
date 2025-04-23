// src/core/state/group_invites/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap, DefaultMemoryImpl, StableVec};

    use crate::{core::{state::group_invites::types::{GroupInvite, GroupInviteID, GroupInviteIDList, GroupInviteeID}, types::UserID}, MEMORY_MANAGER};
    
    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

    pub const INVITES_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(28);
    pub const INVITES_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(29);
    pub const USERS_INVITES_LIST_MEMORY_ID: MemoryId = MemoryId::new(30);

    thread_local! {
        // Convert HashMap to StableBTreeMap for invites by ID
        pub(crate) static INVITES_BY_ID_HASHTABLE: RefCell<StableBTreeMap<GroupInviteID, GroupInvite, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(INVITES_BY_ID_MEMORY_ID))
            )
        );
        
        // Add time-ordered list for invites (similar to other collections)
        pub(crate) static INVITES_BY_TIME_LIST: RefCell<StableVec<GroupInviteID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(INVITES_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize INVITES_BY_TIME_LIST")
        );
        
        // Convert HashMap to StableBTreeMap for user invites
        pub(crate) static USERS_INVITES_LIST_HASHTABLE: RefCell<StableBTreeMap<GroupInviteeID, GroupInviteIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(USERS_INVITES_LIST_MEMORY_ID))
            )
        );
    }


    pub fn initialize() {
        // Force thread_locals in this module to initialize
        INVITES_BY_ID_HASHTABLE.with(|_| {});
        INVITES_BY_TIME_LIST.with(|_| {});
        USERS_INVITES_LIST_HASHTABLE.with(|_| {});
    }

}


