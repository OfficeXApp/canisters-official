// src/core/state/disks/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, BTreeMap, DefaultMemoryImpl, StableBTreeMap, StableVec};

    use crate::{core::{api::uuid::generate_uuidv4, state::{directory::{state::state::{folder_uuid_to_metadata, full_folder_path_to_uuid}, types::{DriveFullFilePath, FolderID, FolderRecord}}, disks::types::{Disk, DiskID, DiskTypeEnum}, drives::{state::state::{DRIVE_ID, OWNER_ID}, types::{DriveID, ExternalID}}}, types::{IDPrefix, UserID}}, debug_log, MEMORY_MANAGER};
    
    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    pub const DISKS_MEMORY_ID: MemoryId = MemoryId::new(11);
    pub const DISKS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(12);

    thread_local! {
        // Replace HashMap with StableBTreeMap for disks by ID
        pub(crate) static DISKS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<DiskID, Disk, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DISKS_MEMORY_ID))
            )
        );
        
        // Replace Vec with StableVec for disks by time list
        pub(crate) static DISKS_BY_TIME_LIST: RefCell<StableVec<DiskID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DISKS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize DISKS_BY_TIME_LIST")
        );
    }

    pub fn initialize() {
        // Force thread_locals in this module to initialize
        DISKS_BY_ID_HASHTABLE.with(|_| {});
        DISKS_BY_TIME_LIST.with(|_| {});
    }

    pub fn init_default_disks() {

        debug_log!("Initializing default admin api key...");

        let current_canister_disk_id = DiskID(generate_uuidv4(IDPrefix::Disk));


        let owner_id = OWNER_ID.with(|owner_id| {
            owner_id.borrow().get().clone()
        });
        let (root_folder, trash_folder) = ensure_disk_root_and_trash_folder(
            &current_canister_disk_id.clone(),
            &owner_id,
            &DRIVE_ID.with(|id| id.clone()),
            DiskTypeEnum::IcpCanister
        );

        let default_canister_disk = Disk {
            id: current_canister_disk_id.clone(),
            name: "Default Admin Canister".to_string(),
            disk_type: DiskTypeEnum::IcpCanister,
            private_note: Some("Default Canister Storage. Not recommended as main cloud storage.".to_string()),
            public_note: Some("Default Canister Storage. Not recommended as main cloud storage.".to_string()),
            auth_json: None,
            labels: vec![],
            external_id: Some(ExternalID(ic_cdk::api::id().to_text())),
            external_payload: None,
            root_folder: root_folder,
            trash_folder: trash_folder,
            created_at: ic_cdk::api::time() / 1_000_000,
            endpoint: None,
        };

        DISKS_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(default_canister_disk.id.clone(), default_canister_disk.clone());
        });

        DISKS_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(&default_canister_disk.id.clone());
        });

    }

    // Helper function to create root folder for a disk
    pub fn ensure_disk_root_and_trash_folder(disk_id: &DiskID, owner_id: &UserID, drive_id: &DriveID, disk_type: DiskTypeEnum) -> (FolderID, FolderID) {
        // Root folder path with trailing slash
        let root_path = DriveFullFilePath(format!("{}::/", disk_id.to_string()));
        
        // Get existing or create new root folder
        let root_folder_uuid = if let Some(existing_uuid) = full_folder_path_to_uuid.get(&root_path) {
            existing_uuid.clone()
        } else {
            // Generate UUID with additional entropy for root folder
            let new_uuid = FolderID(generate_uuidv4(IDPrefix::Folder));
            
            let root_folder = FolderRecord {
                id: new_uuid.clone(),
                name: "Root".to_string(),
                parent_folder_uuid: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_directory_path: root_path.clone(),
                labels: Vec::new(),
                created_by: owner_id.clone(),
                created_at: ic_cdk::api::time() / 1_000_000,
                disk_id: disk_id.clone(),
                disk_type: disk_type.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: owner_id.clone(),
                deleted: false,
                drive_id: drive_id.clone(),
                expires_at: -1,
                restore_trash_prior_folder_uuid: None,
                has_sovereign_permissions: true,
                shortcut_to: None,
                external_id: None,
                external_payload: None,
                notes: None,
            };
    
            full_folder_path_to_uuid.insert(root_path, new_uuid.clone());
            folder_uuid_to_metadata.insert(new_uuid.clone(), root_folder);
            new_uuid
        };
    
        // Trash folder path as a subfolder of root
        let trash_path = DriveFullFilePath(format!("{}::.trash/", disk_id.to_string()));
        
        // Get existing or create new trash folder
        let trash_folder_uuid = if let Some(existing_uuid) = full_folder_path_to_uuid.get(&trash_path) {
            existing_uuid.clone()
        } else {
            // Generate UUID with additional entropy for trash folder
            let new_uuid = FolderID(generate_uuidv4(IDPrefix::Folder));
            
            let trash_folder = FolderRecord {
                id: new_uuid.clone(),
                name: "Trash".to_string(),
                parent_folder_uuid: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_directory_path: trash_path.clone(),
                labels: Vec::new(),
                created_by: owner_id.clone(),
                created_at: ic_cdk::api::time() / 1_000_000,
                disk_id: disk_id.clone(),
                disk_type: disk_type.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: owner_id.clone(),
                deleted: false,
                drive_id: drive_id.clone(),
                expires_at: -1,
                restore_trash_prior_folder_uuid: None,
                has_sovereign_permissions: true,
                shortcut_to: None,
                external_id: None,
                external_payload: None,
                notes: None,
            };
    
            full_folder_path_to_uuid.insert(trash_path, new_uuid.clone());
            folder_uuid_to_metadata.insert(new_uuid.clone(), trash_folder);
            
            new_uuid
        };
        
        // Return both folder UUIDs
        (root_folder_uuid, trash_folder_uuid)
    }
}