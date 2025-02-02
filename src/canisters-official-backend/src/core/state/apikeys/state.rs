
// src/core/state/apikeys/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::{core::{state::{apikeys::types::{ApiKey, ApiKeyID, ApiKeyValue}, drive::state::state::OWNER_ID}, types::UserID}, debug_log};

    thread_local! {
        // users pass in api key value, we O(1) lookup the api key id + O(1) lookup the api key
        pub static APIKEYS_BY_VALUE_HASHTABLE: RefCell<HashMap<ApiKeyValue, ApiKeyID>> = RefCell::new(HashMap::new());
        // default is to use the api key id to lookup the api key
        pub static APIKEYS_BY_ID_HASHTABLE: RefCell<HashMap<ApiKeyID, ApiKey>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub static USERS_APIKEYS_HASHTABLE: RefCell<HashMap<UserID, Vec<ApiKeyID>>> = RefCell::new(HashMap::new());
    }

    // Helper functions to get debug string representations
    pub fn debug_apikeys_by_value() -> String {
        APIKEYS_BY_VALUE_HASHTABLE.with(|map| {
            format!("{:#?}", map.borrow())
        })
    }

    pub fn debug_apikeys_by_id() -> String {
        APIKEYS_BY_ID_HASHTABLE.with(|map| {
            format!("{:#?}", map.borrow())
        })
    }

    pub fn debug_users_apikeys() -> String {
        USERS_APIKEYS_HASHTABLE.with(|map| {
            format!("{:#?}", map.borrow())
        })
    }

    // Function to log all state
    pub fn debug_state() -> String {
        format!(
            "State Debug:\n\nAPIKEYS_BY_VALUE:\n{}\n\nAPIKEYS_BY_ID:\n{}\n\nUSERS_APIKEYS:\n{}",
            debug_apikeys_by_value(),
            debug_apikeys_by_id(),
            debug_users_apikeys()
        )
    }

    pub fn init_default_admin_apikey() {

        debug_log!("Initializing default admin api key...");

        let default_key = ApiKey {
            id: ApiKeyID("mock_api_key_id".to_string()),
            value: ApiKeyValue("mock_api_key_value".to_string()),
            user_id: OWNER_ID.with(|owner_id| owner_id.clone()),
            name: "Default Admin Key".to_string(),
            created_at: ic_cdk::api::time(),
            expires_at: -1,
            is_revoked: false,
        };

        debug_log!("Default admin api key: {}", default_key);

        APIKEYS_BY_VALUE_HASHTABLE.with(|map| {
            map.borrow_mut().insert(default_key.value.clone(), default_key.id.clone());
        });

        APIKEYS_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(default_key.id.clone(), default_key.clone());
        });

        USERS_APIKEYS_HASHTABLE.with(|map| {
            map.borrow_mut().insert(default_key.user_id.clone(), vec![default_key.id.clone()]);
        });
    }
}


