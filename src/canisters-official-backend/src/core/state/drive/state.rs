
// src/core/state/drive/state.rs

pub mod state {
    use std::cell::Cell;

    use crate::core::types::{UserID,CanisterID,PublicKeyBLS};

    thread_local! {
        pub static CANISTER_ID: CanisterID = CanisterID(PublicKeyBLS("Anonymous_Canister".to_string()));
        pub static OWNER_ID: UserID = UserID(PublicKeyBLS("Anonymous_Owner".to_string()));
        pub static GLOBAL_UUID_NONCE: Cell<u64> = Cell::new(0);
    }
}


