// src/core/state/directory/state.rs

pub mod state {
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::ops::Deref;

    use ic_stable_structures::memory_manager::MemoryId;
    use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl};

    use crate::core::state::directory::types::FileVersionID;
    use crate::core::state::{
        directory::types::{DriveFullFilePath, FileRecord, FileID, FolderRecord, FolderID}
    };
    use crate::MEMORY_MANAGER;

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    
    // Memory IDs for each data structure
    pub const FOLDER_UUID_TO_METADATA_MEMORY_ID: MemoryId = MemoryId::new(40);
    pub const FILE_UUID_TO_METADATA_MEMORY_ID: MemoryId = MemoryId::new(41);
    pub const FULL_FOLDER_PATH_TO_UUID_MEMORY_ID: MemoryId = MemoryId::new(42);
    pub const FULL_FILE_PATH_TO_UUID_MEMORY_ID: MemoryId = MemoryId::new(43);
    pub const FILE_VERSION_TO_METADATA_MEMORY_ID: MemoryId = MemoryId::new(52);

    // Wrapper types that implement Deref
    pub struct FolderMap;
    pub struct FileMap;
    pub struct FileVersionMap;
    pub struct FolderPathMap;
    pub struct FilePathMap;

    impl FolderMap {
        pub fn get(&self, key: &FolderID) -> Option<FolderRecord> {
            folder_uuid_to_metadata_inner.with(|map| map.borrow().get(key))
        }

        pub fn insert(&self, key: FolderID, value: FolderRecord) {
            folder_uuid_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut StableBTreeMap<FolderID, FolderRecord, Memory>) -> R) -> R {
            folder_uuid_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FolderID) -> bool {
            folder_uuid_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FolderID) -> Option<FolderRecord> {
            folder_uuid_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }

        pub fn with<R>(&self, f: impl FnOnce(&StableBTreeMap<FolderID, FolderRecord, Memory>) -> R) -> R {
            folder_uuid_to_metadata_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FileMap {
        pub fn get(&self, key: &FileID) -> Option<FileRecord> {
            file_uuid_to_metadata_inner.with(|map| map.borrow().get(key))
        }

        pub fn insert(&self, key: FileID, value: FileRecord) {
            file_uuid_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut StableBTreeMap<FileID, FileRecord, Memory>) -> R) -> R {
            file_uuid_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FileID) -> bool {
            file_uuid_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FileID) -> Option<FileRecord> {
            file_uuid_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }
        
        pub fn with<R>(&self, f: impl FnOnce(&StableBTreeMap<FileID, FileRecord, Memory>) -> R) -> R {
            file_uuid_to_metadata_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FileVersionMap {
        pub fn get(&self, key: &FileVersionID) -> Option<FileRecord> {
            file_version_to_metadata_inner.with(|map| map.borrow().get(key))
        }

        pub fn insert(&self, key: FileVersionID, value: FileRecord) {
            file_version_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut StableBTreeMap<FileVersionID, FileRecord, Memory>) -> R) -> R {
            file_version_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FileVersionID) -> bool {
            file_version_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FileVersionID) -> Option<FileRecord> {
            file_version_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }
        
        pub fn with<R>(&self, f: impl FnOnce(&StableBTreeMap<FileVersionID, FileRecord, Memory>) -> R) -> R {
            file_version_to_metadata_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FolderPathMap {
        pub fn get(&self, key: &DriveFullFilePath) -> Option<FolderID> {
            full_folder_path_to_uuid_inner.with(|map| map.borrow().get(key))
        }

        pub fn insert(&self, key: DriveFullFilePath, value: FolderID) {
            full_folder_path_to_uuid_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut StableBTreeMap<DriveFullFilePath, FolderID, Memory>) -> R) -> R {
            full_folder_path_to_uuid_inner.with(|map| f(&mut map.borrow_mut()))
        }

        pub fn contains_key(&self, key: &DriveFullFilePath) -> bool {
            full_folder_path_to_uuid_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &DriveFullFilePath) -> Option<FolderID> {
            full_folder_path_to_uuid_inner.with(|map| map.borrow_mut().remove(key))
        }

        pub fn with<R>(&self, f: impl FnOnce(&StableBTreeMap<DriveFullFilePath, FolderID, Memory>) -> R) -> R {
            full_folder_path_to_uuid_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FilePathMap {
        pub fn get(&self, key: &DriveFullFilePath) -> Option<FileID> {
            full_file_path_to_uuid_inner.with(|map| map.borrow().get(key))
        }

        pub fn insert(&self, key: DriveFullFilePath, value: FileID) {
            full_file_path_to_uuid_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut StableBTreeMap<DriveFullFilePath, FileID, Memory>) -> R) -> R {
            full_file_path_to_uuid_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &DriveFullFilePath) -> bool {
            full_file_path_to_uuid_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &DriveFullFilePath) -> Option<FileID> {
            full_file_path_to_uuid_inner.with(|map| map.borrow_mut().remove(key))
        }

        pub fn with<R>(&self, f: impl FnOnce(&StableBTreeMap<DriveFullFilePath, FileID, Memory>) -> R) -> R {
            full_file_path_to_uuid_inner.with(|map| f(&map.borrow()))
        }
    }

    // Private thread_local storage
    thread_local! {
        // Replace HashMap with StableBTreeMap for folders by ID
        static folder_uuid_to_metadata_inner: RefCell<StableBTreeMap<FolderID, FolderRecord, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(FOLDER_UUID_TO_METADATA_MEMORY_ID))
            )
        );
        
        // Replace HashMap with StableBTreeMap for files by ID
        static file_uuid_to_metadata_inner: RefCell<StableBTreeMap<FileID, FileRecord, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(FILE_UUID_TO_METADATA_MEMORY_ID))
            )
        );

        // Replace HashMap with StableBTreeMap for file versions by ID
        static file_version_to_metadata_inner: RefCell<StableBTreeMap<FileVersionID, FileRecord, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(FILE_VERSION_TO_METADATA_MEMORY_ID))
            )
        );
        
        // Replace HashMap with StableBTreeMap for folder paths to IDs
        static full_folder_path_to_uuid_inner: RefCell<StableBTreeMap<DriveFullFilePath, FolderID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(FULL_FOLDER_PATH_TO_UUID_MEMORY_ID))
            )
        );
        
        // Replace HashMap with StableBTreeMap for file paths to IDs
        static full_file_path_to_uuid_inner: RefCell<StableBTreeMap<DriveFullFilePath, FileID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(FULL_FILE_PATH_TO_UUID_MEMORY_ID))
            )
        );
    }

    // Public instances with original names
    pub static folder_uuid_to_metadata: FolderMap = FolderMap;
    pub static file_uuid_to_metadata: FileMap = FileMap;
    pub static full_folder_path_to_uuid: FolderPathMap = FolderPathMap;
    pub static full_file_path_to_uuid: FilePathMap = FilePathMap;
    pub static file_version_to_metadata: FileVersionMap = FileVersionMap;

    pub fn initialize() {
        // Force thread_locals in this module to initialize
        folder_uuid_to_metadata_inner.with(|_| {});
        file_uuid_to_metadata_inner.with(|_| {});
        full_folder_path_to_uuid_inner.with(|_| {});
        full_file_path_to_uuid_inner.with(|_| {});
        file_version_to_metadata_inner.with(|_| {});
    }
}

