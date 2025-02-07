// src/rest/directorys/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::state::directory::types::{FileMetadata, FolderMetadata}, rest::webhooks::types::SortDirection};

#[derive(Debug, Clone, Deserialize)]
pub enum DirectoryAction {
    #[serde(rename = "get")]
    Get,
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
    #[serde(rename = "copy")]
    Copy,
    #[serde(rename = "move")]
    Move,
    #[serde(rename = "sync")]
    Sync,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectoryActionRequest {
    pub action: DirectoryAction,
    #[serde(flatten)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryActionResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct UploadChunkResponse {
    pub chunk_id: String,
    pub bytes_received: usize
}

#[derive(Debug, Clone, Deserialize)] 
pub struct CompleteUploadRequest {
    pub file_id: String,
    pub filename: String
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteUploadResponse {
    pub file_id: String,
    pub size: usize,
    pub chunks: u32,
    pub filename: String
}


#[derive(serde::Serialize)]
pub struct FileMetadataResponse {
    pub file_id: String,
    pub total_size: usize,
    pub total_chunks: u32,
    pub filename: String
}

pub type SearchDirectoryResponse = DirectoryListResponse;

pub type DirectoryResponse<'a, T> = crate::rest::drives::types::DriveResponse<'a, T>;
pub type ErrorResponse<'a> = DirectoryResponse<'a, ()>;
