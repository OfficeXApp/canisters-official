use std::fmt;

// src/core/state/directory/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::core::{state::{disks::types::{DiskID, DiskTypeEnum}, drives::types::{ExternalID, ExternalPayload}, tags::types::TagStringValue}, types::{ICPPrincipalString, UserID}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct FolderUUID(pub String);
impl fmt::Display for FolderUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct FileUUID(pub String);
impl fmt::Display for FileUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveFullFilePath(pub String);
impl fmt::Display for DriveFullFilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SerdeDiff)]
pub struct FolderMetadata {
    pub(crate) id: FolderUUID,
    pub(crate) name: String,
    pub(crate) parent_folder_uuid: Option<FolderUUID>,
    pub(crate) subfolder_uuids: Vec<FolderUUID>,
    pub(crate) file_uuids: Vec<FileUUID>,
    pub(crate) full_folder_path: DriveFullFilePath,
    pub(crate) tags: Vec<TagStringValue>,
    pub(crate) created_by: UserID,
    pub(crate) created_date_ms: u64, // unix ms
    pub(crate) last_updated_date_ms: u64,  // unix ms
    pub(crate) last_updated_by: UserID,
    pub(crate) disk_id: DiskID,
    pub(crate) deleted: bool,
    pub(crate) expires_at: i64,
    pub(crate) canister_id: ICPPrincipalString,
    pub(crate) restore_trash_prior_folder_path: Option<DriveFullFilePath>,
    pub(crate) has_sovereign_permissions: bool,
    pub(crate) external_id: Option<ExternalID>,
    pub(crate) external_payload: Option<ExternalPayload>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SerdeDiff)]
pub struct FileMetadata {
    pub(crate) id: FileUUID,
    pub(crate) name: String,
    pub(crate) folder_uuid: FolderUUID,
    pub(crate) file_version: u32,
    pub(crate) prior_version: Option<FileUUID>,
    pub(crate) next_version: Option<FileUUID>,
    pub(crate) extension: String,
    pub(crate) full_file_path: DriveFullFilePath,
    pub(crate) tags: Vec<TagStringValue>,
    pub(crate) created_by: UserID,
    pub(crate) created_date_ms: u64, // unix ms
    pub(crate) disk_id: DiskID,
    pub(crate) disk_type: DiskTypeEnum,
    pub(crate) file_size: u64,
    pub(crate) raw_url: String,
    pub(crate) last_updated_date_ms: u64,  // unix ms
    pub(crate) last_updated_by: UserID,
    pub(crate) deleted: bool,
    pub(crate) canister_id: ICPPrincipalString,
    pub(crate) expires_at: i64,
    pub(crate) restore_trash_prior_folder_path: Option<DriveFullFilePath>,
    pub(crate) has_sovereign_permissions: bool,
    pub(crate) external_id: Option<ExternalID>,
    pub(crate) external_payload: Option<ExternalPayload>,
}




#[derive(Serialize, Deserialize, Debug, SerdeDiff)]
pub struct PathTranslationResponse {
    pub folder: Option<FolderMetadata>,
    pub file: Option<FileMetadata>,
}




#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ShareTrackID(pub String);

impl fmt::Display for ShareTrackID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum ShareTrackResourceID {
    File(FileUUID),
    Folder(FolderUUID)
}

impl fmt::Display for ShareTrackResourceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShareTrackResourceID::File(id) => write!(f, "{}", id),
            ShareTrackResourceID::Folder(id) => write!(f, "{}", id),
        }
    }
}