// src/core/state/permissions/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet};

    use crate::core::{
        state::permissions::types::{
            DirectoryPermission, DirectoryPermissionID, DirectoryGranteeID
        },
    };
    use crate::rest::directory::types::DirectoryResourceID;

    thread_local! {
        // Main storage
        pub(crate) static PERMISSIONS_BY_ID_HASHTABLE: RefCell<HashMap<DirectoryPermissionID, DirectoryPermission>> = 
            RefCell::new(HashMap::new());

        // Resource-based indices for O(1) lookups
        pub(crate) static PERMISSIONS_BY_RESOURCE_HASHTABLE: RefCell<HashMap<DirectoryResourceID, HashSet<DirectoryPermissionID>>> =
            RefCell::new(HashMap::new());

        // Grantee-based indices for O(1) lookups
        pub(crate) static GRANTEE_PERMISSIONS_HASHTABLE: RefCell<HashMap<DirectoryGranteeID, HashSet<DirectoryPermissionID>>> =
            RefCell::new(HashMap::new());

        // Time-based indices (also used for history of one-time links)
        pub(crate) static PERMISSIONS_BY_TIME_LIST: RefCell<Vec<DirectoryPermissionID>> = 
            RefCell::new(Vec::new());
    }

}