// src/core/state/contacts/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::{core::{state::{contacts::types::Contact, drives::state::state::OWNER_ID}, types::{ICPPrincipalString, PublicKeyICP, UserID}}, debug_log};
    
    thread_local! {
        // default is to use the api key id to lookup the api key
        pub(crate) static CONTACTS_BY_ID_HASHTABLE: RefCell<HashMap<UserID, Contact>> = RefCell::new(HashMap::new());
        // default is to use the api key id to lookup the api key
        pub(crate) static CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: RefCell<HashMap<ICPPrincipalString, UserID>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub(crate) static CONTACTS_BY_TIME_LIST: RefCell<Vec<UserID>> = RefCell::new(Vec::new());
        // superswap userid history
        // HISTORY_SUPERSWAP_USERID: HashMap<OldUserID, CurrentUserID>
        pub(crate) static HISTORY_SUPERSWAP_USERID: RefCell<HashMap<UserID, UserID>> = RefCell::new(HashMap::new());
    }

    pub fn init_default_owner_contact() {
        debug_log!("Initializing default owner contact...");

        let owner_id = OWNER_ID.with(|id| id.borrow().clone());
        let default_icp_principal = ICPPrincipalString(PublicKeyICP("".to_string())); // Empty string as placeholder

        let default_contact = Contact {
            id: owner_id.clone(),
            name: "Anonymous Owner".to_string(),
            avatar: None,
            email: None,
            webhook_url: None,
            public_note: "Default system owner".to_string(),
            private_note: None,
            evm_public_address: "".to_string(), // Empty string as placeholder
            icp_principal: default_icp_principal.clone(),
            seed_phrase: None,
            teams: vec![],
            tags: vec![],
            past_user_ids: vec![],
            external_id: None,
            external_payload: None,
            from_placeholder_user_id: None,
            redeem_token: None,
            created_at: ic_cdk::api::time() / 1_000_000,
            last_online_ms: 0,
        };

        debug_log!("Default owner contact: {:?}", default_contact);

        CONTACTS_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(owner_id.clone(), default_contact.clone());
        });

        CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|map| {
            map.borrow_mut().insert(default_icp_principal, owner_id.clone());
        });

        CONTACTS_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(owner_id);
        });
    }
}

