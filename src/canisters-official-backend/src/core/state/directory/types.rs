use std::{borrow::Cow, fmt};

use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
// src/core/state/directory/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::{core::{api::permissions::{directory::{check_directory_permissions, derive_directory_breadcrumbs}, system::check_system_permissions}, state::{disks::types::{DiskID, DiskTypeEnum}, drives::{state::state::OWNER_ID, types::{DriveID, ExternalID, ExternalPayload}}, labels::types::{redact_label, LabelStringValue}, permissions::types::{DirectoryPermissionType, PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, raw_storage::types::UploadStatus}, types::{ICPPrincipalString, UserID}}, rest::directory::types::{DirectoryResourceID, FileRecordFE, FolderRecordFE}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct FolderID(pub String);
impl fmt::Display for FolderID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Storable for FolderID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FolderID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FolderID")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct FileID(pub String);
impl fmt::Display for FileID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Storable for FileID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FileID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FileID")
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct FileVersionID(pub String);
impl fmt::Display for FileVersionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Storable for FileVersionID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FileVersionID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FileVersionID")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct DriveFullFilePath(pub String);
impl fmt::Display for DriveFullFilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Storable for DriveFullFilePath {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DriveFullFilePath");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DriveFullFilePath")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct DriveClippedFilePath(pub String);
impl fmt::Display for DriveClippedFilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Storable for DriveClippedFilePath {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DriveClippedFilePath");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DriveClippedFilePath")
    }
}




#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SerdeDiff, CandidType)]
pub struct FolderRecord {
    pub(crate) id: FolderID,
    pub(crate) name: String,
    pub(crate) parent_folder_uuid: Option<FolderID>,
    pub(crate) subfolder_uuids: Vec<FolderID>,
    pub(crate) file_uuids: Vec<FileID>,
    pub(crate) full_directory_path: DriveFullFilePath,
    pub(crate) labels: Vec<LabelStringValue>,
    pub(crate) created_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub(crate) created_at: u64, // unix ms
    pub(crate) last_updated_date_ms: u64,  // unix ms
    pub(crate) last_updated_by: UserID,  // wont get updated by superswap, reverse loopup HISTORY_SUPERSWAP_USERID
    pub(crate) disk_id: DiskID,
    pub(crate) disk_type: DiskTypeEnum,
    pub(crate) deleted: bool,
    pub(crate) expires_at: i64,
    pub(crate) drive_id: DriveID,
    pub(crate) restore_trash_prior_folder_uuid: Option<FolderID>,
    pub(crate) has_sovereign_permissions: bool,
    pub(crate) shortcut_to: Option<FolderID>,
    pub(crate) external_id: Option<ExternalID>,
    pub(crate) external_payload: Option<ExternalPayload>,
    pub(crate) notes: Option<String>,
}

impl Storable for FolderRecord {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FolderRecord");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FolderRecord")
    }
}


impl FolderRecord {


    pub async fn cast_fe(&self, user_id: &UserID) -> FolderRecordFE {
        let mut folder = self.clone();
        
        // Get user's system permissions for this contact record
        let resource_id = DirectoryResourceID::Folder(folder.id.clone());
        let permission_previews = check_directory_permissions(
            resource_id.clone(),
            PermissionGranteeID::User(user_id.clone()),
        ).await;

        let path_parts = folder.full_directory_path.0.split("/").collect::<Vec<&str>>();
        let mut clipped_path = String::new();
        if path_parts.len() > 1 {
            clipped_path.push_str(path_parts[0]);
            clipped_path.push_str("::");
            if path_parts.len() > 2 {
                clipped_path.push_str("..");
                clipped_path.push_str("/");
            }
            clipped_path.push_str(path_parts[path_parts.len()-1]);
        } else {
            clipped_path.push_str(&folder.full_directory_path.0);
        }

        folder.full_directory_path = DriveFullFilePath("".to_string());


        FolderRecordFE {
            folder,
            clipped_directory_path: DriveClippedFilePath(clipped_path),
            permission_previews,
        }.redacted(user_id)
    }

    
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SerdeDiff, CandidType)]
pub struct FileRecord {
    pub(crate) id: FileID,
    pub(crate) name: String,
    pub(crate) parent_folder_uuid: FolderID,
    pub(crate) file_version: u32,
    pub(crate) prior_version: Option<FileVersionID>,
    pub(crate) next_version: Option<FileVersionID>,
    pub(crate) version_id: FileVersionID,
    pub(crate) extension: String,
    pub(crate) full_directory_path: DriveFullFilePath,
    pub(crate) labels: Vec<LabelStringValue>,
    pub(crate) created_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub(crate) created_at: u64, // unix ms
    pub(crate) disk_id: DiskID,
    pub(crate) disk_type: DiskTypeEnum,
    pub(crate) file_size: u64,
    pub(crate) raw_url: String,
    pub(crate) last_updated_date_ms: u64,  // unix ms
    pub(crate) last_updated_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub(crate) deleted: bool,
    pub(crate) drive_id: DriveID,
    pub(crate) upload_status: UploadStatus,
    pub(crate) expires_at: i64,
    pub(crate) restore_trash_prior_folder_uuid: Option<FolderID>,
    pub(crate) has_sovereign_permissions: bool,
    pub(crate) shortcut_to: Option<FileID>,
    pub(crate) external_id: Option<ExternalID>,
    pub(crate) external_payload: Option<ExternalPayload>,
    pub(crate) notes: Option<String>,
}

impl Storable for FileRecord {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FileRecord");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FileRecord")
    }
}

impl FileRecord {
  
    pub async fn cast_fe(&self, user_id: &UserID) -> FileRecordFE {
        let mut file = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id == &*owner_id.borrow().get());

        // Get user's system permissions for this contact record
        let resource_id = DirectoryResourceID::File(file.id.clone());
        let permission_previews = if is_owner {
            [
                DirectoryPermissionType::View,
                DirectoryPermissionType::Edit,
                DirectoryPermissionType::Delete,
                DirectoryPermissionType::Invite,
                DirectoryPermissionType::Manage
            ].to_vec()
        } else {
            check_directory_permissions(
                resource_id.clone(),
                PermissionGranteeID::User(user_id.clone()),
            ).await
        };

        let path_parts = file.full_directory_path.0.split("/").collect::<Vec<&str>>();
        let mut clipped_path = String::new();
        if path_parts.len() > 1 {
            clipped_path.push_str(path_parts[0]);
            clipped_path.push_str("::");
            if path_parts.len() > 2 {
                clipped_path.push_str("..");
                clipped_path.push_str("/");
            }
            clipped_path.push_str(path_parts[path_parts.len()-1]);
        } else {
            clipped_path.push_str(&file.full_directory_path.0);
        }

        file.full_directory_path = DriveFullFilePath("".to_string());

        FileRecordFE {
            file,
            clipped_directory_path: DriveClippedFilePath(clipped_path),
            permission_previews,
        }.redacted(user_id)
    }

    
}





#[derive(Serialize, Deserialize, Debug, SerdeDiff, CandidType)]
pub struct PathTranslationResponse {
    pub folder: Option<FolderRecord>,
    pub file: Option<FileRecord>,
}




#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct ShareTrackID(pub String);

impl fmt::Display for ShareTrackID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
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