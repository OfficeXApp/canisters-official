
// src/core/state/apikeys/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::state::apikeys::types::ApiKeyItem;

    thread_local! {
        pub static NEXT_APIKEY_ID: RefCell<u32> = RefCell::new(0);
        pub static APIKEY_ITEMS: RefCell<HashMap<u32, ApiKeyItem>> = RefCell::new(HashMap::new());
    }

    pub struct ApiKeyState {
        pub next_id: u32,
        pub items: HashMap<u32, ApiKeyItem>,
    }
    
    impl Default for ApiKeyState {
        fn default() -> Self {
            Self {
                next_id: 0,
                items: HashMap::new(),
            }
        }
    }
}

