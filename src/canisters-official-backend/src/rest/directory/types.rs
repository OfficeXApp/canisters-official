use std::{collections::HashMap, fmt};

// src/rest/directory/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::{api::disks::aws_s3::S3UploadResponse, state::directory::types::{DriveFullFilePath, FileMetadata, FileUUID, FolderMetadata, FolderUUID, Tag}}, rest::webhooks::types::SortDirection};
use crate::core::{
    state::disks::types::{DiskID, DiskTypeEnum},
    types::{ICPPrincipalString, UserID}
};


#[derive(Debug, Clone, Deserialize)]
pub struct SearchDirectoryRequest {
    pub query_string: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListDirectoryRequest {
    pub folder_id: Option<String>,
    pub path: Option<String>,
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListResponse {
    pub folders: Vec<FolderMetadata>,
    pub files: Vec<FileMetadata>,
    pub total_files: usize,
    pub total_folders: usize,
    pub cursor: Option<String>,
}

fn default_page_size() -> usize {
    50
}


#[derive(Debug, Clone, Deserialize)]
pub struct UploadChunkRequest {
    pub file_id: String,
    pub chunk_index: u32,
    pub chunk_data: Vec<u8>,
    pub total_chunks: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadChunkResponse {
    pub chunk_id: String,
    pub bytes_received: usize
}

#[derive(Debug, Clone, Deserialize)] 
pub struct CompleteUploadRequest {
    pub file_id: String,
    pub filename: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUploadResponse {
    pub file_id: String,
    pub size: usize,
    pub chunks: u32,
    pub filename: String
}


#[derive(serde::Serialize, Deserialize)]
pub struct FileMetadataResponse {
    pub file_id: String,
    pub total_size: usize,
    pub total_chunks: u32,
    pub filename: String
}

pub type SearchDirectoryResponse = DirectoryListResponse;

pub type DirectoryResponse<'a, T> = crate::rest::drives::types::DriveResponse<'a, T>;
pub type ErrorResponse<'a> = DirectoryResponse<'a, ()>;



#[derive(Debug, Clone, Deserialize)] 
pub struct ClientSideUploadRequest {
    pub disk_id: String,
    pub folder_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSideUploadResponse {
    pub signature: String,
}


// --------------------------------------------


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryAction {
    pub action: DirectoryActionEnum,
    pub target: ResourceIdentifier,
    pub payload: DirectoryActionPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DirectoryActionOutcomeID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryActionOutcome {
    pub id: DirectoryActionOutcomeID,
    pub success: bool,
    pub action: DirectoryActionEnum,
    pub target: ResourceIdentifier,
    pub payload: DirectoryActionPayload,
    pub result: Option<DirectoryActionResult>,
    pub error: Option<DirectoryActionError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryActionRequestBody {
    pub actions: Vec<DirectoryAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryActionError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone,Serialize,  Deserialize)]
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



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIdentifier {
    #[serde(default)]
    pub resource_path: Option<DriveFullFilePath>, // points to file/folder itself, except in create file/folder operations would be a parent folder
    #[serde(default)]
    pub resource_id: Option<String>,  // points to file/folder itself, except in create file/folder operations would be a parent folder
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFilePayload {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFolderPayload {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFilePayload {
    pub name: String,
    pub extension: String,
    pub tags: Vec<Tag>,
    pub file_size: u64,
    pub raw_url: String,
    pub disk_id: DiskID,
    pub expires_at: Option<i64>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderPayload {
    pub name: String,
    pub tags: Vec<Tag>,
    pub disk_id: DiskID,
    pub expires_at: Option<i64>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFilePayload {
    pub name: Option<String>,
    pub tags: Option<Vec<Tag>>,
    pub raw_url: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFolderPayload {
    pub name: Option<String>,
    pub tags: Option<Vec<Tag>>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFilePayload {
    pub permanent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFolderPayload {
    pub permanent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFilePayload {
    pub destination_folder_id: Option<FolderUUID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFolderPayload {
    pub destination_folder_id: Option<FolderUUID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct MoveFilePayload {
    pub destination_folder_id: Option<FolderUUID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFolderPayload {
    pub destination_folder_id: Option<FolderUUID>,
    pub destination_folder_path: Option<DriveFullFilePath>,
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
}



// Response types remain the same as before
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DirectoryActionResult {
    GetFile(FileMetadata),
    GetFolder(FolderMetadata),
    CreateFile(CreateFileResponse),
    CreateFolder(FolderMetadata),
    UpdateFile(FileMetadata),
    UpdateFolder(FolderMetadata),
    DeleteFile(DeleteFileResponse),
    DeleteFolder(DeleteFolderResponse),
    CopyFile(FileMetadata),
    CopyFolder(FolderMetadata),
    MoveFile(FileMetadata),
    MoveFolder(FolderMetadata),
    RestoreTrash(RestoreTrashResponse)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileResponse {
    pub file: FileMetadata,
    pub upload: S3UploadResponse,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderResponse {
    pub notes: String,
    pub folder: FolderMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileResponse {
    pub file_id: FileUUID,
    pub trash_full_path: DriveFullFilePath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFolderResponse {
    pub folder_id: FolderUUID,
    pub trash_full_path: DriveFullFilePath, // if empty then its permanently deleted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_files: Option<Vec<FileUUID>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_folders: Option<Vec<FolderUUID>>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreTrashPayload {
    pub file_conflict_resolution: Option<FileConflictResolutionEnum>,
    pub restore_to_folder: Option<FolderUUID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreTrashResponse {
    pub restored_files: Vec<FileUUID>,
    pub restored_folders: Vec<FolderUUID>,
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
        "tags": ["work", "2024"],
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
        "tags": ["project", "active"],
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
        "tags": ["work", "2024", "reviewed"],
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
        "tags": ["project", "active", "phase-2"],
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