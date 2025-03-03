// src/core/state/directory/state.rs

pub mod state {
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::ops::Deref;

    use crate::core::state::{
        directory::types::{DriveFullFilePath, FileRecord, FileID, FolderRecord, FolderID},
        templates::types::{TemplateID, TemplateItem},
    };

    // Wrapper types that implement Deref
    pub struct FolderMap;
    pub struct FileMap;
    pub struct FolderPathMap;
    pub struct FilePathMap;

    impl FolderMap {
        pub fn get(&self, key: &FolderID) -> Option<FolderRecord> {
            folder_uuid_to_metadata_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: FolderID, value: FolderRecord) {
            folder_uuid_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<FolderID, FolderRecord>) -> R) -> R {
            folder_uuid_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FolderID) -> bool {
            folder_uuid_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FolderID) -> Option<FolderRecord> {
            folder_uuid_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }

        pub fn with<R>(&self, f: impl FnOnce(&HashMap<FolderID, FolderRecord>) -> R) -> R {
            folder_uuid_to_metadata_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FileMap {
        pub fn get(&self, key: &FileID) -> Option<FileRecord> {
            file_uuid_to_metadata_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: FileID, value: FileRecord) {
            file_uuid_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<FileID, FileRecord>) -> R) -> R {
            file_uuid_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FileID) -> bool {
            file_uuid_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FileID) -> Option<FileRecord> {
            file_uuid_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }
        
        pub fn with<R>(&self, f: impl FnOnce(&HashMap<FileID, FileRecord>) -> R) -> R {
            file_uuid_to_metadata_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FolderPathMap {
        pub fn get(&self, key: &DriveFullFilePath) -> Option<FolderID> {
            full_folder_path_to_uuid_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: DriveFullFilePath, value: FolderID) {
            full_folder_path_to_uuid_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<DriveFullFilePath, FolderID>) -> R) -> R {
            full_folder_path_to_uuid_inner.with(|map| f(&mut map.borrow_mut()))
        }

        pub fn contains_key(&self, key: &DriveFullFilePath) -> bool {
            full_folder_path_to_uuid_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &DriveFullFilePath) -> Option<FolderID> {
            full_folder_path_to_uuid_inner.with(|map| map.borrow_mut().remove(key))
        }

        pub fn with<R>(&self, f: impl FnOnce(&HashMap<DriveFullFilePath, FolderID>) -> R) -> R {
            full_folder_path_to_uuid_inner.with(|map| f(&map.borrow()))
        }
    }

    impl FilePathMap {
        pub fn get(&self, key: &DriveFullFilePath) -> Option<FileID> {
            full_file_path_to_uuid_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: DriveFullFilePath, value: FileID) {
            full_file_path_to_uuid_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<DriveFullFilePath, FileID>) -> R) -> R {
            full_file_path_to_uuid_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &DriveFullFilePath) -> bool {
            full_file_path_to_uuid_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &DriveFullFilePath) -> Option<FileID> {
            full_file_path_to_uuid_inner.with(|map| map.borrow_mut().remove(key))
        }

        pub fn with<R>(&self, f: impl FnOnce(&HashMap<DriveFullFilePath, FileID>) -> R) -> R {
            full_file_path_to_uuid_inner.with(|map| f(&map.borrow()))
        }
    }

    // Private thread_local storage
    thread_local! {
        static folder_uuid_to_metadata_inner: RefCell<HashMap<FolderID, FolderRecord>> = RefCell::new(HashMap::new());
        static file_uuid_to_metadata_inner: RefCell<HashMap<FileID, FileRecord>> = RefCell::new(HashMap::new());
        static full_folder_path_to_uuid_inner: RefCell<HashMap<DriveFullFilePath, FolderID>> = RefCell::new(HashMap::new());
        static full_file_path_to_uuid_inner: RefCell<HashMap<DriveFullFilePath, FileID>> = RefCell::new(HashMap::new());
    }

    // Public instances with original names
    pub static folder_uuid_to_metadata: FolderMap = FolderMap;
    pub static file_uuid_to_metadata: FileMap = FileMap;
    pub static full_folder_path_to_uuid: FolderPathMap = FolderPathMap;
    pub static full_file_path_to_uuid: FilePathMap = FilePathMap;
}

