
// src/rest/directory/types.rs
use std::{borrow::Cow, collections::HashMap, fmt};
use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize, Deserializer, Serializer, ser::SerializeStruct};
use crate::{core::{state::{directory::types::{DriveClippedFilePath, DriveFullFilePath, FileID, FileRecord, FolderID, FolderRecord}, drives::state::state::OWNER_ID, labels::{state::validate_uuid4_string_with_prefix, types::{redact_label, LabelStringValue}}, permissions::types::{DirectoryPermissionID, DirectoryPermissionType, SystemPermissionType}, raw_storage::types::UploadStatus}, types::{ClientSuggestedUUID, IDPrefix}}, rest::{types::{validate_external_id, validate_external_payload, validate_id_string, validate_short_string, validate_unclaimed_uuid, validate_url_endpoint, ValidationError}, webhooks::types::SortDirection}};
use crate::core::{
    state::disks::types::{DiskID, DiskTypeEnum},
    types::{ICPPrincipalString, UserID}
};
use serde::de;
use serde_json::Value;
use serde_diff::{SerdeDiff};




#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct FileRecordFE {
    #[serde(flatten)] 
    pub file: FileRecord,
    pub clipped_directory_path: DriveClippedFilePath,
    pub permission_previews: Vec<DirectoryPermissionType>, 
}

impl FileRecordFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&DirectoryPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                // redact fields
            }
        }
        // Filter labels
        redacted.file.labels = match is_owner {
            true => redacted.file.labels,
            false => redacted.file.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}




#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct FolderRecordFE {
    #[serde(flatten)] 
    pub folder: FolderRecord,
    pub clipped_directory_path: DriveClippedFilePath,
    pub permission_previews: Vec<DirectoryPermissionType>, 
}

impl FolderRecordFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&DirectoryPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                // redact fields
            }
        }
        // Filter labels
        redacted.folder.labels = match is_owner {
            true => redacted.folder.labels,
            false => redacted.folder.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}




#[derive(Debug, Clone, Deserialize, CandidType)]
pub struct SearchDirectoryRequest {
    pub query_string: String,
}
impl SearchDirectoryRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate query_string
        if self.query_string.is_empty() {
            return Err(ValidationError {
                field: "query_string".to_string(),
                message: "Query string cannot be empty".to_string(),
            });
        }
        
        if self.query_string.len() > 256 {
            return Err(ValidationError {
                field: "query_string".to_string(),
                message: "Query string must be 256 characters or less".to_string(),
            });
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ListDirectoryRequest {
    pub folder_id: Option<String>,
    pub path: Option<String>,
    pub disk_id: Option<String>,
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

// Add validation for ListDirectoryRequest
impl ListDirectoryRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate folder_id if provided
        if let Some(folder_id) = &self.folder_id {
            validate_id_string(folder_id, "folder_id")?;
        }

        if let Some(disk_id) = &self.disk_id {
            validate_id_string(disk_id, "disk_id")?;
        }
        
        // Validate path if provided
        if let Some(path) = &self.path {
            if path.len() > 4096 {
                return Err(ValidationError {
                    field: "path".to_string(),
                    message: "Path must be 4,096 characters or less".to_string(),
                });
            }
        }
        
        // Validate filters
        if self.filters.len() > 256 {
            return Err(ValidationError {
                field: "filters".to_string(),
                message: "Filters must be 256 characters or less".to_string(),
            });
        }
        
        // Validate page_size
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(ValidationError {
                field: "page_size".to_string(),
                message: "Page size must be between 1 and 1000".to_string(),
            });
        }
        
        // Validate cursor if provided
        if let Some(cursor) = &self.cursor {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DirectoryListResponse {
    pub folders: Vec<FolderRecordFE>,
    pub files: Vec<FileRecordFE>,
    pub total_files: usize,
    pub total_folders: usize,
    pub cursor: Option<String>,
    pub breadcrumbs: Vec<FilePathBreadcrumb>,
    pub permission_previews: Vec<DirectoryPermissionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct FilePathBreadcrumb {
    pub resource_id: String,
    pub resource_name: String,
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DiskUploadResponse {
    pub url: String,
    pub fields: HashMap<String, String>,
}


#[derive(Debug, Clone, Deserialize, CandidType)]
pub struct UploadChunkRequest {
    pub file_id: String,
    pub chunk_index: u32,
    pub chunk_data: Vec<u8>,
    pub total_chunks: u32
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UploadChunkResponse {
    pub chunk_id: String,
    pub bytes_received: usize
}

#[derive(Debug, Clone, Deserialize, CandidType)] 
pub struct CompleteUploadRequest {
    pub file_id: String,
    pub filename: String
}
impl CompleteUploadRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate file_id
        validate_id_string(&self.file_id, "file_id")?;
        
        // Validate filename
        validate_short_string(&self.filename, "filename")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CompleteUploadResponse {
    pub file_id: String,
    pub size: usize,
    pub chunks: u32,
    pub filename: String
}


#[derive(serde::Serialize, Deserialize, CandidType)]
pub struct FileMetadataResponse {
    pub file_id: String,
    pub total_size: usize,
    pub total_chunks: u32,
    pub filename: String
}

pub type SearchDirectoryResponse = DirectoryListResponse;

pub type DirectoryResponse<'a, T> = crate::rest::types::ApiResponse<'a, T>;
pub type ErrorResponse<'a> = DirectoryResponse<'a, ()>;



#[derive(Debug, Clone, Deserialize, CandidType)] 
pub struct ClientSideUploadRequest {
    pub disk_id: String,
    pub folder_path: String,
}
impl ClientSideUploadRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate disk_id
        validate_id_string(&self.disk_id, "disk_id")?;
        
        // Validate folder_path
        if self.folder_path.len() > 4096 {
            return Err(ValidationError {
                field: "folder_path".to_string(),
                message: "Folder path must be 4,096 characters or less".to_string(),
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ClientSideUploadResponse {
    pub signature: String,
}


// --------------------------------------------


#[derive(Debug, Clone)]
pub struct DirectoryAction {
    pub action: DirectoryActionEnum,
    pub payload: DirectoryActionPayload,
}
impl DirectoryAction {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        
        // Validate payload based on action type
        match self.action {
            DirectoryActionEnum::GetFile => {
                match &self.payload {
                    DirectoryActionPayload::GetFile(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for GET_FILE action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::GetFolder => {
                match &self.payload {
                    DirectoryActionPayload::GetFolder(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for GET_FOLDER action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::CreateFile => {
                match &self.payload {
                    DirectoryActionPayload::CreateFile(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for CREATE_FILE action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::CreateFolder => {
                match &self.payload {
                    DirectoryActionPayload::CreateFolder(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for CREATE_FOLDER action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::UpdateFile => {
                match &self.payload {
                    DirectoryActionPayload::UpdateFile(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for UPDATE_FILE action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::UpdateFolder => {
                match &self.payload {
                    DirectoryActionPayload::UpdateFolder(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for UPDATE_FOLDER action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::DeleteFile => {
                match &self.payload {
                    DirectoryActionPayload::DeleteFile(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for DELETE_FILE action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::DeleteFolder => {
                match &self.payload {
                    DirectoryActionPayload::DeleteFolder(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for DELETE_FOLDER action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::CopyFile => {
                match &self.payload {
                    DirectoryActionPayload::CopyFile(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for COPY_FILE action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::CopyFolder => {
                match &self.payload {
                    DirectoryActionPayload::CopyFolder(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for COPY_FOLDER action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::MoveFile => {
                match &self.payload {
                    DirectoryActionPayload::MoveFile(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for MOVE_FILE action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::MoveFolder => {
                match &self.payload {
                    DirectoryActionPayload::MoveFolder(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for MOVE_FOLDER action".to_string(),
                    }),
                }
            },
            DirectoryActionEnum::RestoreTrash => {
                match &self.payload {
                    DirectoryActionPayload::RestoreTrash(payload) => payload.validate_body()?,
                    _ => return Err(ValidationError {
                        field: "payload".to_string(),
                        message: "Invalid payload type for RESTORE_TRASH action".to_string(),
                    }),
                }
            },
        }
        
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct RawDirectoryAction {
    action: DirectoryActionEnum,
    payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
pub struct DirectoryActionOutcomeID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryActionOutcome {
    pub id: DirectoryActionOutcomeID,
    pub success: bool,
    pub request: DirectoryAction,
    pub response: DirectoryActionResponse,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryActionResponse {
    pub result: Option<DirectoryActionResult>,
    pub error: Option<DirectoryActionError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryActionRequestBody {
    pub actions: Vec<DirectoryAction>,
}
impl DirectoryActionRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate each action in the list
        for (i, action) in self.actions.iter().enumerate() {
            match action.validate_body() {
                Ok(_) => continue,
                Err(e) => return Err(ValidationError {
                    field: format!("actions[{}].{}", i, e.field),
                    message: e.message,
                }),
            }
        }
        
        // Validate that there's at least one action
        if self.actions.is_empty() {
            return Err(ValidationError {
                field: "actions".to_string(),
                message: "At least one action must be provided".to_string(),
            });
        }
        
        Ok(())
    }
}

// Custom deserialization for DirectoryAction.
impl<'de> Deserialize<'de> for DirectoryAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawDirectoryAction::deserialize(deserializer)?;
        // Dispatch based on the action enum to convert the raw JSON payload.
        let payload = match raw.action {
            DirectoryActionEnum::GetFile => {
                DirectoryActionPayload::GetFile(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::GetFolder => {
                DirectoryActionPayload::GetFolder(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::CreateFile => {
                DirectoryActionPayload::CreateFile(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::CreateFolder => {
                DirectoryActionPayload::CreateFolder(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::UpdateFile => {
                DirectoryActionPayload::UpdateFile(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::UpdateFolder => {
                DirectoryActionPayload::UpdateFolder(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::DeleteFile => {
                DirectoryActionPayload::DeleteFile(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::DeleteFolder => {
                DirectoryActionPayload::DeleteFolder(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::CopyFile => {
                DirectoryActionPayload::CopyFile(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::CopyFolder => {
                DirectoryActionPayload::CopyFolder(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::MoveFile => {
                DirectoryActionPayload::MoveFile(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::MoveFolder => {
                DirectoryActionPayload::MoveFolder(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
            DirectoryActionEnum::RestoreTrash => {
                DirectoryActionPayload::RestoreTrash(serde_json::from_value(raw.payload)
                    .map_err(de::Error::custom)?)
            }
        };

        Ok(DirectoryAction {
            action: raw.action,
            payload,
        })
    }
}

// Custom serialization for DirectoryAction.
impl Serialize for DirectoryAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DirectoryAction", 3)?;
        state.serialize_field("action", &self.action)?;
        // Match on the payload variant so that it serializes as a plain JSON object.
        match &self.payload {
            DirectoryActionPayload::GetFile(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::GetFolder(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::CreateFile(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::CreateFolder(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::UpdateFile(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::UpdateFolder(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::DeleteFile(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::DeleteFolder(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::CopyFile(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::CopyFolder(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::MoveFile(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::MoveFolder(p) => state.serialize_field("payload", p)?,
            DirectoryActionPayload::RestoreTrash(p) => state.serialize_field("payload", p)?,
        }
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DirectoryActionError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectoryActionEnum {
    GetFile,
    GetFolder,
    CreateFile,
    CreateFolder,
    UpdateFile,
    UpdateFolder,
    DeleteFile,
    DeleteFolder,
    CopyFile,
    CopyFolder,
    MoveFile,
    MoveFolder,
    RestoreTrash,
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, CandidType)]
pub enum FileConflictResolutionEnum {
    REPLACE,
    KEEP_BOTH,
    KEEP_ORIGINAL,
    KEEP_NEWER,
}
impl fmt::Display for FileConflictResolutionEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileConflictResolutionEnum::REPLACE => write!(f, "REPLACE"),
            FileConflictResolutionEnum::KEEP_BOTH => write!(f, "KEEP_BOTH"),
            FileConflictResolutionEnum::KEEP_ORIGINAL => write!(f, "KEEP_ORIGINAL"),
            FileConflictResolutionEnum::KEEP_NEWER => write!(f, "KEEP_NEWER"),
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub enum DirectoryResourceID {
    File(FileID),
    Folder(FolderID),
}
impl Storable for DirectoryResourceID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DirectoryResourceID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DirectoryResourceID")
    }
}
impl fmt::Display for DirectoryResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectoryResourceID::File(id) => write!(f, "{}", id),
            DirectoryResourceID::Folder(id) => write!(f, "{}", id),
        }
    }
}
impl DirectoryResourceID {
    fn from_string(id: String) -> Option<Self> {
        if id.starts_with(IDPrefix::File.as_str()) {
            Some(DirectoryResourceID::File(FileID(id)))
        } else if id.starts_with(IDPrefix::Folder.as_str()) {
            Some(DirectoryResourceID::Folder(FolderID(id)))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum DirectoryActionPayload {
    GetFile(GetFilePayload),
    GetFolder(GetFolderPayload),
    CreateFile(CreateFilePayload),
    CreateFolder(CreateFolderPayload),
    UpdateFile(UpdateFilePayload),
    UpdateFolder(UpdateFolderPayload),
    DeleteFile(DeleteFilePayload),
    DeleteFolder(DeleteFolderPayload),
    CopyFile(CopyFilePayload),
    CopyFolder(CopyFolderPayload),
    MoveFile(MoveFilePayload),
    MoveFolder(MoveFolderPayload),
    RestoreTrash(RestoreTrashPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct GetFilePayload {
    pub id: FileID,
    pub share_track_hash: Option<String>,
}
impl GetFilePayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;
        // Validate share_track_hash if provided
        if let Some(share_track_hash) = &self.share_track_hash {
            if share_track_hash.len() > 256 {
                return Err(ValidationError {
                    field: "share_track_hash".to_string(),
                    message: "Share track hash must be 256 characters or less".to_string(),
                });
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct GetFolderPayload {
    pub id: FolderID,
    pub share_track_hash: Option<String>,
}
impl GetFolderPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;
        // Validate share_track_hash if provided
        if let Some(share_track_hash) = &self.share_track_hash {
            if share_track_hash.len() > 256 {
                return Err(ValidationError {
                    field: "share_track_hash".to_string(),
                    message: "Share track hash must be 256 characters or less".to_string(),
                });
            }
        }
        
        Ok(())
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct CreateFilePayload {
    pub id: Option<ClientSuggestedUUID>, 
    pub name: String,
    pub parent_folder_uuid: FolderID,
    pub extension: String,
    pub labels: Vec<LabelStringValue>,
    pub file_size: u64,
    pub disk_id: DiskID,
    pub disk_type: DiskTypeEnum,
    pub expires_at: Option<i64>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
    pub has_sovereign_permissions: Option<bool>,
    pub shortcut_to: Option<FileID>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateFilePayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {


        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::File)?;
        }

        validate_uuid4_string_with_prefix(&self.parent_folder_uuid.to_string(), IDPrefix::Folder)?;
        
        // Validate name
        validate_short_string(&self.name, "name")?;
        
        // Validate extension
        if self.extension.len() > 20 {
            return Err(ValidationError {
                field: "extension".to_string(),
                message: "File extension must be 20 characters or less".to_string(),
            });
        }
        
        // Validate labels
        for label in &self.labels {
            if label.0.len() > 256 {
                return Err(ValidationError {
                    field: "labels".to_string(),
                    message: "Each label must be 256 characters or less".to_string(),
                });
            }
        }
        
        // Validate disk_id
        validate_id_string(&self.disk_id.0, "disk_id")?;
        
        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }
        
        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }
        
        Ok(())
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct CreateFolderPayload {
    pub id: Option<ClientSuggestedUUID>,
    pub name: String,
    pub parent_folder_uuid: FolderID,
    pub labels: Vec<LabelStringValue>,
    pub disk_id: DiskID,
    pub disk_type: Option<DiskTypeEnum>,
    pub expires_at: Option<i64>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
    pub has_sovereign_permissions: Option<bool>,
    pub shortcut_to: Option<FolderID>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateFolderPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {


        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::Folder)?;
        }

        // Validate name
        validate_short_string(&self.name, "name")?;
        validate_uuid4_string_with_prefix(&self.parent_folder_uuid.to_string(), IDPrefix::Folder)?;
        
        // Validate labels
        for label in &self.labels {
            if label.0.len() > 256 {
                return Err(ValidationError {
                    field: "labels".to_string(),
                    message: "Each label must be 256 characters or less".to_string(),
                });
            }
        }
        
        // Validate disk_id
        validate_id_string(&self.disk_id.0, "disk_id")?;
        
        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }
        
        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct UpdateFilePayload {
    pub id: FileID,
    pub name: Option<String>,
    pub labels: Option<Vec<LabelStringValue>>,
    pub expires_at: Option<i64>,
    pub shortcut_to: Option<FileID>,
    pub upload_status: Option<UploadStatus>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl UpdateFilePayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // validate id
        validate_id_string(&self.id.0, "id")?;

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_short_string(name, "name")?;
        }
        
        // Validate labels if provided
        if let Some(labels) = &self.labels {
            for label in labels {
                if label.0.len() > 256 {
                    return Err(ValidationError {
                        field: "labels".to_string(),
                        message: "Each label must be 256 characters or less".to_string(),
                    });
                }
            }
        }
        
        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }
        
        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct UpdateFolderPayload {
    pub id: FolderID,
    pub name: Option<String>,
    pub labels: Option<Vec<LabelStringValue>>,
    pub expires_at: Option<i64>,
    pub shortcut_to: Option<FolderID>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl UpdateFolderPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        // Validate id
        validate_id_string(&self.id.0, "id")?;

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_short_string(name, "name")?;
        }
        
        // Validate labels if provided
        if let Some(labels) = &self.labels {
            for label in labels {
                if label.0.len() > 256 {
                    return Err(ValidationError {
                        field: "labels".to_string(),
                        message: "Each label must be 256 characters or less".to_string(),
                    });
                }
            }
        }
        
        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }
        
        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct DeleteFilePayload {
    pub id: FileID,
    pub permanent: bool,
}
impl DeleteFilePayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;

        // Nothing to validate for this simple payload
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct DeleteFolderPayload {
    pub id: FolderID,
    pub permanent: bool,
}
impl DeleteFolderPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;
        // Nothing to validate for this simple payload
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct CopyFilePayload {
    pub id: FileID,
    pub destination_folder_id: Option<FolderID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
    pub new_copy_id: Option<ClientSuggestedUUID>,
}
impl CopyFilePayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        // Validate id
        validate_id_string(&self.id.0, "id")?;

        if self.new_copy_id.is_some() {
            validate_unclaimed_uuid(&self.new_copy_id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.new_copy_id.as_ref().unwrap().to_string(), IDPrefix::File)?;
        }

        // Validate destination_folder_id if provided
        if let Some(folder_id) = &self.destination_folder_id {
            validate_id_string(&folder_id.0, "destination_folder_id")?;
        }
        
        // Validate destination_folder_path if provided
        if let Some(folder_path) = &self.destination_folder_path {
            if folder_path.0.len() > 4096 {
                return Err(ValidationError {
                    field: "destination_folder_path".to_string(),
                    message: "Destination folder path must be 4,096 characters or less".to_string(),
                });
            }
        }
        
        // At least one of destination_folder_id or destination_folder_path must be provided
        if self.destination_folder_id.is_none() && self.destination_folder_path.is_none() {
            return Err(ValidationError {
                field: "destination".to_string(),
                message: "Either destination_folder_id or destination_folder_path must be provided".to_string(),
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct CopyFolderPayload {
    pub id: FolderID,
    pub destination_folder_id: Option<FolderID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
    pub new_copy_id: Option<ClientSuggestedUUID>,
}
impl CopyFolderPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        // Validate id
        validate_id_string(&self.id.0, "id")?;

        if self.new_copy_id.is_some() {
            validate_unclaimed_uuid(&self.new_copy_id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.new_copy_id.as_ref().unwrap().to_string(), IDPrefix::Folder)?;
        }

        // Validate destination_folder_id if provided
        if let Some(folder_id) = &self.destination_folder_id {
            validate_id_string(&folder_id.0, "destination_folder_id")?;
        }
        
        // Validate destination_folder_path if provided
        if let Some(folder_path) = &self.destination_folder_path {
            if folder_path.0.len() > 4096 {
                return Err(ValidationError {
                    field: "destination_folder_path".to_string(),
                    message: "Destination folder path must be 4,096 characters or less".to_string(),
                });
            }
        }
        
        // At least one of destination_folder_id or destination_folder_path must be provided
        if self.destination_folder_id.is_none() && self.destination_folder_path.is_none() {
            return Err(ValidationError {
                field: "destination".to_string(),
                message: "Either destination_folder_id or destination_folder_path must be provided".to_string(),
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct MoveFilePayload {
    pub id: FileID,
    pub destination_folder_id: Option<FolderID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}
impl MoveFilePayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;

        // Validate destination_folder_id if provided
        if let Some(folder_id) = &self.destination_folder_id {
            validate_id_string(&folder_id.0, "destination_folder_id")?;
        }
        
        // Validate destination_folder_path if provided
        if let Some(folder_path) = &self.destination_folder_path {
            if folder_path.0.len() > 4096 {
                return Err(ValidationError {
                    field: "destination_folder_path".to_string(),
                    message: "Destination folder path must be 4,096 characters or less".to_string(),
                });
            }
        }
        
        // At least one of destination_folder_id or destination_folder_path must be provided
        if self.destination_folder_id.is_none() && self.destination_folder_path.is_none() {
            return Err(ValidationError {
                field: "destination".to_string(),
                message: "Either destination_folder_id or destination_folder_path must be provided".to_string(),
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct MoveFolderPayload {
    pub id: FolderID,
    pub destination_folder_id: Option<FolderID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}
impl MoveFolderPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;

        // Validate destination_folder_id if provided
        if let Some(folder_id) = &self.destination_folder_id {
            validate_id_string(&folder_id.0, "destination_folder_id")?;
        }
        
        // Validate destination_folder_path if provided
        if let Some(folder_path) = &self.destination_folder_path {
            if folder_path.0.len() > 4096 {
                return Err(ValidationError {
                    field: "destination_folder_path".to_string(),
                    message: "Destination folder path must be 4,096 characters or less".to_string(),
                });
            }
        }
        
        // At least one of destination_folder_id or destination_folder_path must be provided
        if self.destination_folder_id.is_none() && self.destination_folder_path.is_none() {
            return Err(ValidationError {
                field: "destination".to_string(),
                message: "Either destination_folder_id or destination_folder_path must be provided".to_string(),
            });
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(deny_unknown_fields)]
pub struct RestoreTrashPayload {
    pub id: String, // FileID or FolderID
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
    pub restore_to_folder_path: Option<DriveFullFilePath>,
}
impl RestoreTrashPayload {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id, "id")?;

        // Validate restore_to_folder_path if provided
        if let Some(folder_path) = &self.restore_to_folder_path {
            if folder_path.0.len() > 4096 {
                return Err(ValidationError {
                    field: "restore_to_folder_path".to_string(),
                    message: "Restore to folder path must be 4,096 characters or less".to_string(),
                });
            }
        }
        
        Ok(())
    }
}




#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct GetFileResponse {
    pub file: FileRecordFE,
    pub breadcrumbs: Vec<FilePathBreadcrumb>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct GetFolderResponse {
    pub folder: FolderRecordFE,
    pub breadcrumbs: Vec<FilePathBreadcrumb>,
}


// Response types remain the same as before
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
#[serde(untagged)]
pub enum DirectoryActionResult {
    GetFile(GetFileResponse),
    GetFolder(GetFolderResponse),
    CreateFile(CreateFileResponse),
    CreateFolder(CreateFolderResponse),
    UpdateFile(FileRecordFE),
    UpdateFolder(FolderRecordFE),
    DeleteFile(DeleteFileResponse),
    DeleteFolder(DeleteFolderResponse),
    CopyFile(FileRecordFE),
    CopyFolder(FolderRecordFE),
    MoveFile(FileRecordFE),
    MoveFolder(FolderRecordFE),
    RestoreTrash(RestoreTrashResponse)
}




#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CreateFileResponse {
    pub file: FileRecordFE,
    pub upload: DiskUploadResponse,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CreateFolderResponse {
    pub notes: String,
    pub folder: FolderRecordFE,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DeleteFileResponse {
    pub file_id: FileID,
    pub path_to_trash: DriveFullFilePath,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DeleteFolderResponse {
    pub folder_id: FolderID,
    pub path_to_trash: DriveFullFilePath, // if empty then its permanently deleted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_files: Option<Vec<FileID>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_folders: Option<Vec<FolderID>>,
}


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RestoreTrashResponse {
    pub restored_files: Vec<FileID>,
    pub restored_folders: Vec<FolderID>,
}

#[derive(Serialize, Deserialize, Clone, Debug, CandidType)]
pub struct DirectoryResourcePermissionFE {
    pub permission_id: String,
    pub grant_type: String,
}

// Example JSON requests:
/*
1. GET_FILE request (by path):
{
    "action": "GET_FILE",
    "target": {
        "resource_path": "/user/documents/report.pdf"
    },
    "payload": {}
}

2. GET_FILE request (by id):
{
    "action": "GET_FILE",
    "target": {
        "resource_id": "file-uuid-123"
    },
    "payload": {}
}

3. GET_FOLDER request:
{
    "action": "GET_FOLDER",
    "target": {
        "resource_id": "folder-uuid-456"
    },
    "payload": {}
}

4. CREATE_FILE request:
{
    "action": "CREATE_FILE",
    "target": {
        "resource_path": "/user/documents/report.pdf"
    },
    "payload": {
        "name": "report.pdf",
        "folder_uuid": "folder-uuid-789",
        "extension": "pdf",
        "labels": ["work", "2024"],
        "file_size": 1024567,
        "raw_url": "https://example.com/files/raw/123",
        "disk_id": "disk-1",
        "expires_at": 1735689600000
    }
}

5. CREATE_FOLDER request:
{
    "action": "CREATE_FOLDER",
    "target": {
        "resource_path": "/user/documents/project-alpha"
    },
    "payload": {
        "name": "project-alpha",
        "parent_folder_uuid": "folder-uuid-123",
        "labels": ["project", "active"],
        "disk_id": "disk-1",
        "expires_at": 1735689600000
    }
}

6. UPDATE_FILE request:
{
    "action": "UPDATE_FILE",
    "target": {
        "resource_id": "file-uuid-123"
    },
    "payload": {
        "name": "updated-report.pdf",
        "folder_uuid": "folder-uuid-new",
        "labels": ["work", "2024", "reviewed"],
        "raw_url": "https://example.com/files/raw/124",
        "expires_at": 1735689600000
    }
}

7. UPDATE_FOLDER request:
{
    "action": "UPDATE_FOLDER",
    "target": {
        "resource_id": "folder-uuid-456"
    },
    "payload": {
        "name": "project-beta",
        "parent_folder_uuid": "folder-uuid-new-parent",
        "labels": ["project", "active", "phase-2"],
        "expires_at": 1735689600000
    }
}

8. DELETE_FILE request:
{
    "action": "DELETE_FILE",
    "target": {
        "resource_id": "file-uuid-123"
    },
    "payload": {
        "permanent": false
    }
}

9. DELETE_FOLDER request:
{
    "action": "DELETE_FOLDER",
    "target": {
        "resource_id": "folder-uuid-456"
    },
    "payload": {
        "permanent": false,
        "recursive": true
    }
}

10. COPY_FILE request:
{
    "action": "COPY_FILE",
    "target": {
        "resource_id": "file-uuid-123"
    },
    "payload": {
        "destination_folder_id": "folder-uuid-destination",
        "new_name": "report-copy.pdf"
    }
}

11. COPY_FOLDER request:
{
    "action": "COPY_FOLDER",
    "target": {
        "resource_id": "folder-uuid-456"
    },
    "payload": {
        "destination_parent_id": "folder-uuid-destination",
        "new_name": "project-alpha-backup",
        "recursive": true
    }
}

12. MOVE_FILE request:
{
    "action": "MOVE_FILE",
    "target": {
        "resource_id": "file-uuid-123"
    },
    "payload": {
        "destination_folder_id": "folder-uuid-destination",
        "new_name": "report-new-location.pdf"
    }
}

13. MOVE_FOLDER request:
{
    "action": "MOVE_FOLDER",
    "target": {
        "resource_id": "folder-uuid-456"
    },
    "payload": {
        "destination_parent_id": "folder-uuid-destination",
        "new_name": "project-alpha-archived"
    }
}

14. RESTORE_TRASH request:
{
    "action": "RESTORE_TRASH",
    "target": {
        "resource_id": "folder-uuid-456"
    },
*/