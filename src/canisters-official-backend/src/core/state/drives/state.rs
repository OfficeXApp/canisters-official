
// src/core/state/drives/state.rs

pub mod state {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::drives::types::Drive;
    use crate::core::state::drives::types::DriveID;
    use crate::core::types::{UserID,PublicKeyBLS};

    thread_local! {
        // self info
        pub static CANISTER_ID: DriveID = DriveID(PublicKeyBLS(ic_cdk::api::id().to_text()));
        pub static OWNER_ID: UserID = UserID("Anonymous_Owner".to_string());
        pub static GLOBAL_UUID_NONCE: Cell<u64> = Cell::new(0);
        // hashtables
        pub static DRIVES_BY_ID_HASHTABLE: RefCell<HashMap<DriveID, Drive>> = RefCell::new(HashMap::new());
        pub static DRIVES_BY_TIME_LIST: RefCell<Vec<DriveID>> = RefCell::new(Vec::new());
    }

    pub fn init_self_drive() {
        let self_drive = Drive  {
            id: DriveID(PublicKeyBLS(ic_cdk::api::id().to_text())),
            name: "Anonymous_Canister".to_string(),
            owner_id: Some(UserID("Anonymous_Owner".to_string())),
            gas_remaining: Some(ic_cdk::api::canister_balance()),
            public_note: Some("".to_string()),
            private_note: Some("".to_string()),
        };

        DRIVES_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(self_drive.id.clone(), self_drive.clone());
        });

        DRIVES_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(self_drive.id.clone());
        });
    }
}


