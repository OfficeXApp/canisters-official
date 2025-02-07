// src/core/state/disks/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::disks::types::{DiskID, Disk};
    
    thread_local! {
        pub(crate) static DISKS_BY_ID_HASHTABLE: RefCell<HashMap<DiskID, Disk>> = RefCell::new(HashMap::new());
        pub(crate) static DISKS_BY_EXTERNAL_ID_HASHTABLE: RefCell<HashMap<String, DiskID>> = RefCell::new(HashMap::new());
        pub(crate) static DISKS_BY_TIME_LIST: RefCell<Vec<DiskID>> = RefCell::new(Vec::new());
    }

}


