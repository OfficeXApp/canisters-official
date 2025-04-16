// src/core/state/contacts/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, BTreeMap, StableBTreeMap, StableVec, DefaultMemoryImpl, Vec};

    use crate::{core::{state::{contacts::types::{Contact, ContactIDList}, drives::state::state::OWNER_ID}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, MEMORY_MANAGER};
    
    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    pub const CONTACTS_MEMORY_ID: MemoryId = MemoryId::new(7); 
    pub const CONTACTS_BY_ICP_MEMORY_ID: MemoryId = MemoryId::new(8);
    pub const CONTACTS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(9);
    pub const HISTORY_SUPERSWAP_MEMORY_ID: MemoryId = MemoryId::new(10);

    thread_local! {
        // Replace HashMap with StableBTreeMap for contacts by ID
        pub(crate) static CONTACTS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<UserID, Contact, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(CONTACTS_MEMORY_ID))
            )
        );
        
        // Replace HashMap with StableBTreeMap for contacts by ICP principal
        pub(crate) static CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: RefCell<StableBTreeMap<ICPPrincipalString, UserID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(CONTACTS_BY_ICP_MEMORY_ID))
            )
        );
        
        // Replace Vec with StableVec for contacts by time list
        pub(crate) static CONTACTS_BY_TIME_LIST: RefCell<StableVec<UserID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(CONTACTS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize CONTACTS_BY_TIME_LIST")
        );
        
        // Replace HashMap with StableBTreeMap for superswap history
        pub(crate) static HISTORY_SUPERSWAP_USERID: RefCell<StableBTreeMap<UserID, UserID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(HISTORY_SUPERSWAP_MEMORY_ID))
            )
        );
    }


    pub fn init_default_owner_contact(name: Option<String>) {
        debug_log!("Initializing default owner contact...");

        let owner_id = OWNER_ID.with(|id| id.borrow().get().clone());
        // extract icp principal by removing the prefix IDPrefix::User
        let owner_icp_principal = owner_id.to_icp_principal_string();

        let default_name = match name {
            Some(name) => name,
            None => "Anonymous Owner".to_string(),
        };

        let default_contact = Contact {
            id: owner_id.clone(),
            name: default_name,
            avatar: None,
            email: None,
            notifications_url: None,
            public_note: Some("Default system owner".to_string()),
            private_note: None,
            evm_public_address: "".to_string(), // Empty string as placeholder
            icp_principal: owner_icp_principal.clone(),
            seed_phrase: None,
            groups: vec![],
            labels: vec![],
            past_user_ids: vec![],
            external_id: None,
            external_payload: None,
            from_placeholder_user_id: None,
            redeem_code: None,
            created_at: ic_cdk::api::time() / 1_000_000,
            last_online_ms: 0,
        };

        debug_log!("Default owner contact: {:?}", default_contact);

        CONTACTS_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(owner_id.clone(), default_contact.clone());
        });

        CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|map| {
            map.borrow_mut().insert(owner_icp_principal, owner_id.clone());
        });

        CONTACTS_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(&owner_id);
        });
    }
}

