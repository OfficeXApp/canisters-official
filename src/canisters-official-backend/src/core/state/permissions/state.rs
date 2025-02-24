// src/core/state/permissions/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::{HashMap};

    use crate::core::state::permissions::types::{SystemPermission, SystemPermissionID, SystemResourceID};
    use crate::core::{
        state::permissions::types::{
            DirectoryPermission, DirectoryPermissionID, PermissionGranteeID
        },
    };
    use crate::rest::directory::types::DirectoryResourceID;

    thread_local! {
        // Main storage
        pub(crate) static DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE: RefCell<HashMap<DirectoryPermissionID, DirectoryPermission>> = 
            RefCell::new(HashMap::new());

        // Resource-based indices for O(1) lookups
        pub(crate) static DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE: RefCell<HashMap<DirectoryResourceID, Vec<DirectoryPermissionID>>> =
            RefCell::new(HashMap::new());

        // Grantee-based indices for O(1) lookups
        pub(crate) static DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE: RefCell<HashMap<PermissionGranteeID, Vec<DirectoryPermissionID>>> =
            RefCell::new(HashMap::new());

        // Time-based indices (also used for history of one-time links)
        pub(crate) static DIRECTORY_PERMISSIONS_BY_TIME_LIST: RefCell<Vec<DirectoryPermissionID>> = 
            RefCell::new(Vec::new());


        // Main storage for system permissions
        pub(crate) static SYSTEM_PERMISSIONS_BY_ID_HASHTABLE: RefCell<HashMap<SystemPermissionID, SystemPermission>> = 
        RefCell::new(HashMap::new());

        // Resource-based indices for O(1) lookups
        pub(crate) static SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE: RefCell<HashMap<SystemResourceID, Vec<SystemPermissionID>>> =
            RefCell::new(HashMap::new());

        // Grantee-based indices for O(1) lookups
        pub(crate) static SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE: RefCell<HashMap<PermissionGranteeID, Vec<SystemPermissionID>>> =
            RefCell::new(HashMap::new());

        // Time-based indices (also used for history of one-time links)
        pub(crate) static SYSTEM_PERMISSIONS_BY_TIME_LIST: RefCell<Vec<SystemPermissionID>> = 
        RefCell::new(Vec::new());
        
    }

}