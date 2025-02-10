// src/core/state/disks/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::{core::{api::uuid::generate_unique_id, state::{directory::{state::state::{folder_uuid_to_metadata, full_folder_path_to_uuid}, types::{DriveFullFilePath, FolderMetadata, FolderUUID}}, disks::types::{Disk, DiskID, DiskTypeEnum, DEFAULT_BROWSERCACHE_DISK_ID, DEFAULT_CANISTER_DISK_ID}, drives::state::state::{CANISTER_ID, OWNER_ID}}, types::{ICPPrincipalString, PublicKeyBLS, UserID}}, debug_log};
    
    thread_local! {
        pub(crate) static DISKS_BY_ID_HASHTABLE: RefCell<HashMap<DiskID, Disk>> = RefCell::new(HashMap::new());
        pub(crate) static DISKS_BY_EXTERNAL_ID_HASHTABLE: RefCell<HashMap<String, DiskID>> = RefCell::new(HashMap::new());
        pub(crate) static DISKS_BY_TIME_LIST: RefCell<Vec<DiskID>> = RefCell::new(Vec::new());
    }

    pub fn init_default_disks() {

        debug_log!("Initializing default admin api key...");

        let current_canister_disk_id = generate_unique_id("DiskID", &format!("--DiskType_{}", DiskTypeEnum::IcpCanister));
        let default_canister_disk = Disk {
            id: DiskID(current_canister_disk_id.clone()),
            name: "Self Canister Storage (Default)".to_string(),
            disk_type: DiskTypeEnum::IcpCanister,
            private_note: Some("Default Canister Storage".to_string()),
            public_note: Some("Default Canister Storage".to_string()),
            auth_json: None,
            external_id: Some(ic_cdk::api::id().to_text()),
        };
        let browsercache_disk_id = generate_unique_id("DiskID", &format!("--DiskType_{}", DiskTypeEnum::BrowserCache));
        let default_browsercache_disk = Disk {
            id: DiskID(browsercache_disk_id.clone()),
            name: "Ephemeral Browser Storage (Default)".to_string(),
            disk_type: DiskTypeEnum::BrowserCache,
            private_note: Some("Offline web browser cache. Do not expect persistence in case browser history cleared.".to_string()),
            public_note: Some("Offline web browser cache. Do not expect persistence in case browser history cleared.".to_string()),
            auth_json: None,
            external_id: Some(format!("{}_DEFAULT_BROWSERCACHE_DISK_ID",ic_cdk::api::id().to_text())),
        };

        let default_disks = vec![default_canister_disk, default_browsercache_disk];

        for disk in default_disks {
            DISKS_BY_ID_HASHTABLE.with(|map| {
                map.borrow_mut().insert(disk.id.clone(), disk.clone());
            });

            DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|map| {
                map.borrow_mut().insert(disk.external_id.clone().unwrap(), disk.id.clone());
            });

            DISKS_BY_TIME_LIST.with(|list| {
                list.borrow_mut().push(disk.id.clone());
            });

            OWNER_ID.with(|owner_id| {
                ensure_disk_root_folder(
                    &disk.id,
                    &owner_id.clone(),
                    &ic_cdk::api::id().to_text()
                );
            });
        }

    }

    // Helper function to create root folder for a disk
    pub fn ensure_disk_root_folder(disk_id: &DiskID, owner_id: &UserID, canister_id: &str) {
        let root_path = DriveFullFilePath(format!("{}::", disk_id.to_string()));
        
        // Only create if root folder doesn't exist
        if !full_folder_path_to_uuid.contains_key(&root_path) {
            let root_folder_uuid = generate_unique_id("FolderUUID", "");
            let root_folder = FolderMetadata {
                id: FolderUUID(root_folder_uuid.clone()),
                name: String::new(),
                parent_folder_uuid: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_folder_path: root_path.clone(),
                tags: Vec::new(),
                created_by: owner_id.clone(),
                created_date_ms: ic_cdk::api::time(),
                disk_id: disk_id.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: owner_id.clone(),
                deleted: false,
                canister_id: ICPPrincipalString(PublicKeyBLS(canister_id.to_string())),
                expires_at: -1,
            };

            full_folder_path_to_uuid.insert(root_path, FolderUUID(root_folder_uuid.clone()));
            folder_uuid_to_metadata.insert(FolderUUID(root_folder_uuid), root_folder);
        }
    }
}


