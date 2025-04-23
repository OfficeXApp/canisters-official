// src/core/state/webhooks/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap, StableVec, DefaultMemoryImpl};

    use crate::{core::state::webhooks::types::{Webhook, WebhookAltIndexID, WebhookID, WebhookIDList}, MEMORY_MANAGER};

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    
    // Define memory IDs for each stable structure
    pub const WEBHOOKS_BY_ALT_INDEX_MEMORY_ID: MemoryId = MemoryId::new(37);
    pub const WEBHOOKS_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(38);
    pub const WEBHOOKS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(39);
    
    thread_local! {
        // Convert HashMap<WebhookAltIndexID, Vec<WebhookID>> to StableBTreeMap<WebhookAltIndexID, WebhookIDList>
        pub(crate) static WEBHOOKS_BY_ALT_INDEX_HASHTABLE: RefCell<StableBTreeMap<WebhookAltIndexID, WebhookIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(WEBHOOKS_BY_ALT_INDEX_MEMORY_ID))
            )
        );
        
        // Convert HashMap<WebhookID, Webhook> to StableBTreeMap<WebhookID, Webhook>
        pub(crate) static WEBHOOKS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<WebhookID, Webhook, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(WEBHOOKS_BY_ID_MEMORY_ID))
            )
        );
        
        // Convert Vec<WebhookID> to StableVec<WebhookID>
        pub(crate) static WEBHOOKS_BY_TIME_LIST: RefCell<StableVec<WebhookID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(WEBHOOKS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize WEBHOOKS_BY_TIME_LIST")
        );
    }


    pub fn initialize() {
        // Force thread_locals in this module to initialize
        WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|_| {});
        WEBHOOKS_BY_ID_HASHTABLE.with(|_| {});
        WEBHOOKS_BY_TIME_LIST.with(|_| {});
    }

}
