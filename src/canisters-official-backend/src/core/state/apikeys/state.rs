
// src/core/state/apikeys/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::{state::apikeys::types::{ApiKey, ApiKeyID, ApiKeyValue}, types::UserID};

    thread_local! {
        // users pass in api key value, we O(1) lookup the api key id + O(1) lookup the api key
        pub static HASHTABLE_APIKEYS_BY_VALUE: RefCell<HashMap<ApiKeyValue, ApiKeyID>> = RefCell::new(HashMap::new());
        // default is to use the api key id to lookup the api key
        pub static HASHTABLE_APIKEYS_BY_ID: RefCell<HashMap<ApiKeyID, ApiKey>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub static HASHTABLE_USERS_APIKEYS: RefCell<HashMap<UserID, Vec<ApiKeyID>>> = RefCell::new(HashMap::new());
    }
}


