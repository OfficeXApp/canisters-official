// src/core/state/directory/state.rs

pub mod state {
    use std::cell::{RefCell, RefMut};
    use std::collections::HashMap;
    use std::ops::Deref;

    use crate::core::state::{
        directory::types::{DriveFullFilePath, FileMetadata, FileUUID, FolderMetadata, FolderUUID},
        templates::types::{TemplateID, TemplateItem},
    };

    // Wrapper types that implement Deref
    struct FolderMap;
    struct FileMap;
    struct FolderPathMap;
    struct FilePathMap;

    impl FolderMap {
        pub fn get(&self, key: &FolderUUID) -> Option<FolderMetadata> {
            folder_uuid_to_metadata_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: FolderUUID, value: FolderMetadata) {
            folder_uuid_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<FolderUUID, FolderMetadata>) -> R) -> R {
            folder_uuid_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FolderUUID) -> bool {
            folder_uuid_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FolderUUID) -> Option<FolderMetadata> {
            folder_uuid_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }
    }

    impl FileMap {
        pub fn get(&self, key: &FileUUID) -> Option<FileMetadata> {
            file_uuid_to_metadata_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: FileUUID, value: FileMetadata) {
            file_uuid_to_metadata_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<FileUUID, FileMetadata>) -> R) -> R {
            file_uuid_to_metadata_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &FileUUID) -> bool {
            file_uuid_to_metadata_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &FileUUID) -> Option<FileMetadata> {
            file_uuid_to_metadata_inner.with(|map| map.borrow_mut().remove(key))
        }
    }

    impl FolderPathMap {
        pub fn get(&self, key: &DriveFullFilePath) -> Option<FolderUUID> {
            full_folder_path_to_uuid_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: DriveFullFilePath, value: FolderUUID) {
            full_folder_path_to_uuid_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<DriveFullFilePath, FolderUUID>) -> R) -> R {
            full_folder_path_to_uuid_inner.with(|map| f(&mut map.borrow_mut()))
        }

        pub fn contains_key(&self, key: &DriveFullFilePath) -> bool {
            full_folder_path_to_uuid_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &DriveFullFilePath) -> Option<FolderUUID> {
            full_folder_path_to_uuid_inner.with(|map| map.borrow_mut().remove(key))
        }
    }

    impl FilePathMap {
        pub fn get(&self, key: &DriveFullFilePath) -> Option<FileUUID> {
            full_file_path_to_uuid_inner.with(|map| map.borrow().get(key).cloned())
        }

        pub fn insert(&self, key: DriveFullFilePath, value: FileUUID) {
            full_file_path_to_uuid_inner.with(|map| map.borrow_mut().insert(key, value));
        }

        pub fn with_mut<R>(&self, f: impl FnOnce(&mut HashMap<DriveFullFilePath, FileUUID>) -> R) -> R {
            full_file_path_to_uuid_inner.with(|map| f(&mut map.borrow_mut()))
        }
    
        pub fn contains_key(&self, key: &DriveFullFilePath) -> bool {
            full_file_path_to_uuid_inner.with(|map| map.borrow().contains_key(key))
        }
    
        pub fn remove(&self, key: &DriveFullFilePath) -> Option<FileUUID> {
            full_file_path_to_uuid_inner.with(|map| map.borrow_mut().remove(key))
        }
    }

    // Private thread_local storage
    thread_local! {
        static folder_uuid_to_metadata_inner: RefCell<HashMap<FolderUUID, FolderMetadata>> = RefCell::new(HashMap::new());
        static file_uuid_to_metadata_inner: RefCell<HashMap<FileUUID, FileMetadata>> = RefCell::new(HashMap::new());
        static full_folder_path_to_uuid_inner: RefCell<HashMap<DriveFullFilePath, FolderUUID>> = RefCell::new(HashMap::new());
        static full_file_path_to_uuid_inner: RefCell<HashMap<DriveFullFilePath, FileUUID>> = RefCell::new(HashMap::new());
    }

    // Public instances with original names
    pub static folder_uuid_to_metadata: FolderMap = FolderMap;
    pub static file_uuid_to_metadata: FileMap = FileMap;
    pub static full_folder_path_to_uuid: FolderPathMap = FolderPathMap;
    pub static full_file_path_to_uuid: FilePathMap = FilePathMap;
}