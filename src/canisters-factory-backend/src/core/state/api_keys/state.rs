
// factory repo
// src/core/state/api_keys/state.rs
pub mod state {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use ic_stable_structures::memory_manager::MemoryId;
    use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl, StableVec};

    use crate::core::state::api_keys::types::ApiKeyIDList;
    use crate::MEMORY_MANAGER;
    use crate::{core::{api::uuid::{generate_api_key, generate_uuidv4}, state::{api_keys::types::{ApiKey, ApiKeyID, ApiKeyValue}, giftcards_spawnorg::state::state::OWNER_ID}, types::{IDPrefix, UserID}}, debug_log};

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    pub const APIKEYS_MEMORY_ID: MemoryId = MemoryId::new(1);
    pub const APIKEYS_BY_VALUE_MEMORY_ID: MemoryId = MemoryId::new(2);
    pub const USERS_APIKEYS_MEMORY_ID: MemoryId = MemoryId::new(3);
    pub const APIKEYS_BY_HISTORY_MEMORY_ID: MemoryId = MemoryId::new(4);

    thread_local! {
        // users pass in api key value, we O(1) lookup the api key id + O(1) lookup the api key
        pub(crate) static APIKEYS_BY_VALUE_HASHTABLE: RefCell<StableBTreeMap<ApiKeyValue, ApiKeyID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(APIKEYS_BY_VALUE_MEMORY_ID))
            )
        );
        // default is to use the api key id to lookup the api key
        pub(crate) static APIKEYS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<ApiKeyID, ApiKey, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(APIKEYS_MEMORY_ID))
            )
        );
        // track in hashtable users list of ApiKeyIDs
        pub(crate) static USERS_APIKEYS_HASHTABLE: RefCell<StableBTreeMap<UserID, ApiKeyIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(USERS_APIKEYS_MEMORY_ID))
            )
        );
        // track in vector the history of api keys, similar to CONTACTS_BY_TIME_LIST
        pub(crate) static APIKEYS_BY_HISTORY: RefCell<StableVec<ApiKeyID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(APIKEYS_BY_HISTORY_MEMORY_ID))
            ).expect("Failed to initialize APIKEYS_BY_HISTORY")
        );
    }

    // Helper functions to get debug string representations
    pub fn debug_apikeys_by_value() -> String {
        APIKEYS_BY_VALUE_HASHTABLE.with(|map| {
            let map_ref = map.borrow();
            let mut entries = Vec::new();
            
            for key in map_ref.iter().map(|(k, _)| k) {
                if let Some(value) = map_ref.get(&key) {
                    entries.push(format!("{:?} => {:?}", key, value));
                }
            }
            
            format!("{{\n  {}\n}}", entries.join(",\n  "))
        })
    }

    pub fn debug_apikeys_by_id() -> String {
        APIKEYS_BY_ID_HASHTABLE.with(|map| {
            let map_ref = map.borrow();
            let mut entries = Vec::new();
            
            for key in map_ref.iter().map(|(k, _)| k) {
                if let Some(value) = map_ref.get(&key) {
                    entries.push(format!("{:?} => {:?}", key, value));
                }
            }
            
            format!("{{\n  {}\n}}", entries.join(",\n  "))
        })
    }

    pub fn debug_users_apikeys() -> String {
        USERS_APIKEYS_HASHTABLE.with(|map| {
            let map_ref = map.borrow();
            let mut entries = Vec::new();
            
            for key in map_ref.iter().map(|(k, _)| k) {
                if let Some(value) = map_ref.get(&key) {
                    entries.push(format!("{:?} => {:?}", key, value));
                }
            }
            
            format!("{{\n  {}\n}}", entries.join(",\n  "))
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
            id: ApiKeyID(generate_uuidv4(IDPrefix::ApiKey)),
            value: ApiKeyValue(generate_api_key()),
            user_id: OWNER_ID.with(|id| id.borrow().get().clone()),
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
            let key_list = ApiKeyIDList::with_key(default_key.id.clone());
            map.borrow_mut().insert(default_key.user_id.clone(), key_list);
        });

        APIKEYS_BY_HISTORY.with(|list| {
            list.borrow_mut().push(&default_key.id);
        });
    }
}


