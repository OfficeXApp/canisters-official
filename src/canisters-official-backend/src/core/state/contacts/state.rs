// src/core/state/contacts/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::{state::contacts::types::Contact, types::UserID};
    
    thread_local! {
        // default is to use the api key id to lookup the api key
        pub static CONTACTS_BY_ID_HASHTABLE: RefCell<HashMap<UserID, Contact>> = RefCell::new(HashMap::new());
        // default is to use the api key id to lookup the api key
        pub static CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: RefCell<HashMap<String, UserID>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub static CONTACTS_BY_TIME_LIST: RefCell<Vec<UserID>> = RefCell::new(Vec::new());
    }

}

