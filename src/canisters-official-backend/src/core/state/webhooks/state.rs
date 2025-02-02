// src/core/state/webhooks/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::webhooks::types::{WebhookID, WebhookItem};
    
    thread_local! {
        pub static WEBHOOK_ITEMS: RefCell<HashMap<WebhookID, WebhookItem>> = RefCell::new(HashMap::new());
    }

    pub struct WebhookState {
        pub items: HashMap<WebhookID, WebhookItem>,
    }

    impl Default for WebhookState {
        fn default() -> Self {
            Self {
                items: HashMap::new(),
            }
        }
    }
}


