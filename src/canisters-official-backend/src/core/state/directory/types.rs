use std::fmt;

// src/core/state/directory/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::core::{api::permissions::system::check_system_permissions, state::{disks::types::{DiskID, DiskTypeEnum}, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}}, types::{ICPPrincipalString, UserID}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct FolderID(pub String);
impl fmt::Display for FolderID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct FileID(pub String);
impl fmt::Display for FileID {
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
pub struct FolderRecord {
    pub(crate) id: FolderID,
    pub(crate) name: String,
    pub(crate) parent_folder_uuid: Option<FolderID>,
    pub(crate) subfolder_uuids: Vec<FolderID>,
    pub(crate) file_uuids: Vec<FileID>,
    pub(crate) full_folder_path: DriveFullFilePath,
    pub(crate) tags: Vec<TagStringValue>,
    pub(crate) created_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub(crate) created_at: u64, // unix ms
    pub(crate) last_updated_date_ms: u64,  // unix ms
    pub(crate) last_updated_by: UserID,  // wont get updated by superswap, reverse loopup HISTORY_SUPERSWAP_USERID
    pub(crate) disk_id: DiskID,
    pub(crate) deleted: bool,
    pub(crate) expires_at: i64,
    pub(crate) canister_id: ICPPrincipalString,
    pub(crate) restore_trash_prior_folder_path: Option<DriveFullFilePath>,
    pub(crate) has_sovereign_permissions: bool,
    pub(crate) external_id: Option<ExternalID>,
    pub(crate) external_payload: Option<ExternalPayload>,
}

impl FolderRecord {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();
        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        // Filter tags
        redacted.tags = match is_owner {
            true => redacted.tags,
            false => redacted.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SerdeDiff)]
pub struct FileRecord {
    pub(crate) id: FileID,
    pub(crate) name: String,
    pub(crate) folder_uuid: FolderID,
    pub(crate) file_version: u32,
    pub(crate) prior_version: Option<FileID>,
    pub(crate) next_version: Option<FileID>,
    pub(crate) extension: String,
    pub(crate) full_file_path: DriveFullFilePath,
    pub(crate) tags: Vec<TagStringValue>,
    pub(crate) created_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub(crate) created_at: u64, // unix ms
    pub(crate) disk_id: DiskID,
    pub(crate) disk_type: DiskTypeEnum,
    pub(crate) file_size: u64,
    pub(crate) raw_url: String,
    pub(crate) last_updated_date_ms: u64,  // unix ms
    pub(crate) last_updated_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub(crate) deleted: bool,
    pub(crate) canister_id: ICPPrincipalString,
    pub(crate) expires_at: i64,
    pub(crate) restore_trash_prior_folder_path: Option<DriveFullFilePath>,
    pub(crate) has_sovereign_permissions: bool,
    pub(crate) external_id: Option<ExternalID>,
    pub(crate) external_payload: Option<ExternalPayload>,
}

impl FileRecord {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();
        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        // Filter tags
        redacted.tags = match is_owner {
            true => redacted.tags,
            false => redacted.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}





#[derive(Serialize, Deserialize, Debug, SerdeDiff)]
pub struct PathTranslationResponse {
    pub folder: Option<FolderRecord>,
    pub file: Option<FileRecord>,
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
    File(FileID),
    Folder(FolderID)
}

impl fmt::Display for ShareTrackResourceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShareTrackResourceID::File(id) => write!(f, "{}", id),
            ShareTrackResourceID::Folder(id) => write!(f, "{}", id),
        }
    }
}