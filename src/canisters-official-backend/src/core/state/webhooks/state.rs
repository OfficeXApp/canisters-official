// src/core/state/webhooks/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::webhooks::types::{Webhook, WebhookAltIndexID, WebhookID};
    
    thread_local! {
        // users pass in api key value, we O(1) lookup the api key id + O(1) lookup the api key
        pub static WEBHOOKS_BY_ALT_INDEX_HASHTABLE: RefCell<HashMap<WebhookAltIndexID, WebhookID>> = RefCell::new(HashMap::new());
        // default is to use the api key id to lookup the api key
        pub static WEBHOOKS_BY_ID_HASHTABLE: RefCell<HashMap<WebhookID, Webhook>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub static WEBHOOKS_BY_TIME_LIST: RefCell<Vec<WebhookID>> = RefCell::new(Vec::new());
    }

}


