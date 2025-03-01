
// src/core/state/drives/state.rs

pub mod state {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::api::replay::diff::update_checksum_for_state_diff;
    use crate::core::api::uuid::generate_unique_id;
    use crate::core::state::drives::types::Drive;
    use crate::core::state::drives::types::DriveID;
    use crate::core::state::drives::types::DriveRESTUrlEndpoint;
    use crate::core::state::drives::types::ExternalID;
    use crate::core::state::drives::types::StateChecksum;
    use crate::core::state::drives::types::DriveStateDiffString;
    use crate::core::types::ICPPrincipalString;
    use crate::core::types::IDPrefix;
    use crate::core::types::{UserID,PublicKeyICP};

    thread_local! {
        // self info - immutable
        pub(crate) static DRIVE_ID: DriveID = DriveID(generate_unique_id(IDPrefix::Drive, ""));
        pub(crate) static CANISTER_ID: PublicKeyICP = PublicKeyICP(ic_cdk::api::id().to_text());
        pub(crate) static GLOBAL_UUID_NONCE: Cell<u64> = Cell::new(0);
        pub(crate) static DRIVE_STATE_CHECKSUM: RefCell<StateChecksum> = RefCell::new(StateChecksum("genesis".to_string()));
        pub(crate) static DRIVE_STATE_TIMESTAMP_NS: Cell<u64> = Cell::new(ic_cdk::api::time());
        // self info - mutable
        pub(crate) static OWNER_ID: RefCell<UserID> = RefCell::new(UserID("Anonymous_Owner".to_string()));
        pub(crate) static URL_ENDPOINT: RefCell<DriveRESTUrlEndpoint> = RefCell::new(DriveRESTUrlEndpoint(format!("https://{}.icp0.io", CANISTER_ID.with(|id| id.0.clone()))));
        // hashtables
        pub(crate) static DRIVES_BY_ID_HASHTABLE: RefCell<HashMap<DriveID, Drive>> = RefCell::new(HashMap::new());
        pub(crate) static DRIVES_BY_TIME_LIST: RefCell<Vec<DriveID>> = RefCell::new(Vec::new());
        // external id tracking
        pub(crate) static EXTERNAL_ID_MAPPINGS: RefCell<HashMap<ExternalID, Vec<String>>> = RefCell::new(HashMap::new());
    }

    pub fn init_self_drive() {
        let self_drive = Drive {
            id: DRIVE_ID.with(|id| id.clone()),
            name: "Anonymous_Canister".to_string(),
            public_note: Some("".to_string()),
            private_note: Some("".to_string()),
            icp_principal: ICPPrincipalString(PublicKeyICP(ic_cdk::api::id().to_text())),
            url_endpoint: URL_ENDPOINT.with(|url| url.borrow().clone()),
            last_indexed_ms: None,
            tags: vec![],
        };

        DRIVES_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(self_drive.id.clone(), self_drive.clone());
        });

        DRIVES_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(self_drive.id.clone());
        });

        update_checksum_for_state_diff(DriveStateDiffString("".to_string()));
    }

    pub fn update_external_id_mapping(
        old_external_id: Option<ExternalID>,
        new_external_id: Option<ExternalID>,
        internal_id: Option<String>,
    ) {
        EXTERNAL_ID_MAPPINGS.with(|mappings| {
            let mut mappings_mut = mappings.borrow_mut();
            
            // Handle removal of old external ID mapping if it exists
            if let Some(old_id) = old_external_id {
                if let Some(ids) = mappings_mut.get_mut(&old_id) {
                    // Remove the internal_id from the old mapping
                    ids.retain(|id| id != internal_id.as_ref().unwrap());
                    
                    // If the vector is now empty, remove the mapping entirely
                    if ids.is_empty() {
                        mappings_mut.remove(&old_id);
                    }
                }
            }
            
            // Handle adding new external ID mapping if it exists
            let internal_id = internal_id.unwrap();
            if let Some(new_id) = new_external_id {
                mappings_mut
                    .entry(new_id)
                    .and_modify(|ids| {
                        // Only add if it's not already in the list
                        if !ids.contains(&internal_id) {
                            ids.push(internal_id.clone());
                        }
                    })
                    .or_insert_with(|| vec![internal_id.clone()]);
            }
        });
        
    }

}


