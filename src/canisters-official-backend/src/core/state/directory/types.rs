use std::fmt;

// src/core/state/directory/types.rs
use serde::{Serialize, Deserialize};

use crate::core::{state::disks::types::{DiskID, DiskTypeEnum}, types::{ICPPrincipalString, UserID}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FolderUUID(pub String);
impl fmt::Display for FolderUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileUUID(pub String);
impl fmt::Display for FileUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DriveFullFilePath(pub String);
impl fmt::Display for DriveFullFilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(pub String);





#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FolderMetadata {
    pub id: FolderUUID,
    pub original_folder_name: String,
    pub parent_folder_uuid: Option<FolderUUID>,
    pub subfolder_uuids: Vec<FolderUUID>,
    pub file_uuids: Vec<FileUUID>,
    pub full_folder_path: DriveFullFilePath,
    pub tags: Vec<Tag>,
    pub owner: UserID,
    pub created_date: u64, // unix ns   
    pub disk_id: DiskID,
    pub last_changed_unix_ms: u64,
    pub deleted: bool,
    pub expires_at: i64,
    pub canister_id: ICPPrincipalString,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FileMetadata {
    pub(crate) id: FileUUID,
    pub(crate) original_file_name: String,
    pub(crate) folder_uuid: FolderUUID,
    pub(crate) file_version: u32,
    pub(crate) prior_version: Option<FileUUID>,
    pub(crate) next_version: Option<FileUUID>,
    pub(crate) extension: String,
    pub(crate) full_file_path: DriveFullFilePath,
    pub(crate) tags: Vec<Tag>,
    pub(crate) owner: UserID,
    pub(crate) created_date: u64, // unix ns
    pub(crate) disk_id: DiskID,
    pub(crate) file_size: u64,
    pub(crate) raw_url: String,
    pub(crate) last_changed_unix_ms: u64, 
    pub(crate) deleted: bool,
    pub(crate) canister_id: ICPPrincipalString,
    pub(crate) expires_at: i64,
}
