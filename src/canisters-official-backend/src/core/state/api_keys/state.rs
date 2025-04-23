
// src/core/state/api_keys/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap, DefaultMemoryImpl};
    use crate::{core::{api::uuid::{generate_api_key, generate_uuidv4}, state::{api_keys::types::{ApiKey, ApiKeyID, ApiKeyIDList, ApiKeyValue}, drives::state::state::OWNER_ID}, types::{IDPrefix, UserID}}, debug_log, MEMORY_MANAGER};

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    pub const APIKEYS_MEMORY_ID: MemoryId = MemoryId::new(4);
    pub const APIKEYS_BY_VALUE_MEMORY_ID: MemoryId = MemoryId::new(5);
    pub const USERS_APIKEYS_MEMORY_ID: MemoryId = MemoryId::new(6);

    thread_local! {
        // users pass in api key value, we O(1) lookup the api key id + O(1) lookup the api key
        pub(crate) static APIKEYS_BY_VALUE_HASHTABLE: RefCell<StableBTreeMap<ApiKeyValue, ApiKeyID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(APIKEYS_BY_VALUE_MEMORY_ID))
            )
        );
        // default is to use the api key id to lookup the api key
        // This will replace your HashMap, but keep the same name
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
    }

    pub fn initialize() {
        // Force thread_locals in this module to initialize
        APIKEYS_BY_VALUE_HASHTABLE.with(|_| {});
        APIKEYS_BY_ID_HASHTABLE.with(|_| {});
        USERS_APIKEYS_HASHTABLE.with(|_| {});
    }

    pub fn init_default_admin_apikey() {

        debug_log!("Initializing default admin api key...");

        let default_key = ApiKey {
            id: ApiKeyID(generate_uuidv4(IDPrefix::ApiKey)),
            value: ApiKeyValue(generate_api_key()),
            user_id: OWNER_ID.with(|id| id.borrow().get().clone()),
            name: "Default Admin Key".to_string(),
            private_note: None,
            created_at: ic_cdk::api::time(),
            begins_at: 0,
            expires_at: -1,
            is_revoked: false,
            labels: vec![],
            external_id: None,
            external_payload: None,
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
    }
}


