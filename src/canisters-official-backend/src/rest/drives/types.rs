// src/rest/drives/types.rs

use serde::{Deserialize, Serialize};
use crate::core::state::drives::types::{Drive, DriveID, DriveStateDiffID, StateChecksum, StateDiffRecord};
use crate::core::state::search::types::{SearchCategoryEnum, SearchResult};
use crate::core::types::PublicKeyICP;
use crate::rest::webhooks::types::{SortDirection, WebhookResponse};

#[derive(Debug, Clone, Serialize)]
pub enum DriveResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> DriveResponse<'a, T> {
    pub fn ok(data: &'a T) -> DriveResponse<'a, T> {
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn unauthorized() -> Self {
        Self::err(401, "Unauthorized".to_string())
    }

    pub fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize value")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListDrivesRequestBody {
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize)]
pub struct ListDrivesResponseData {
    pub items: Vec<Drive>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertDriveRequestBody {
    Create(CreateDriveRequestBody),
    Update(UpdateDriveRequestBody),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDriveRequestBody {
    pub name: String,
    pub icp_principal: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub url_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDriveRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icp_principal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_endpoint: Option<String>,
}

pub type GetDriveResponse<'a> = DriveResponse<'a, Drive>;
pub type ListDrivesResponse<'a> = DriveResponse<'a, ListDrivesResponseData>;
pub type CreateDriveResponse<'a> = DriveResponse<'a, Drive>;
pub type UpdateDriveResponse<'a> = DriveResponse<'a, Drive>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDriveRequest {
    pub id: DriveID,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedDriveData {
    pub id: DriveID,
    pub deleted: bool
}

pub type DeleteDriveResponse<'a> = DriveResponse<'a, DeletedDriveData>;
pub type ErrorResponse<'a> = DriveResponse<'a, ()>;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDriveRequestBody {
    pub diffs: Vec<StateDiffRecord>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDriveResponseData {
    pub timestamp_ns: u64,
    pub diffs_applied: usize,
    pub checkpoint_diff_id: Option<DriveStateDiffID>,
    pub final_checksum: StateChecksum,
}

pub type ReplayDriveResponse<'a> = DriveResponse<'a, ReplayDriveResponseData>;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchSortByEnum {
    #[serde(rename = "created_at")]
    CreatedAt,
    #[serde(rename = "updated_at")]
    UpdatedAt,
}

impl Default for SearchSortByEnum {
    fn default() -> Self {
        SearchSortByEnum::UpdatedAt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDriveRequestBody {
    pub query: String,
    #[serde(default)]
    pub categories: Vec<SearchCategoryEnum>,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
    #[serde(default)]
    pub sort_by: SearchSortByEnum,
    #[serde(default)]
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDriveResponseData {
    pub items: Vec<SearchResult>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub type SearchDriveResponse<'a> = DriveResponse<'a, SearchDriveResponseData>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexDriveRequestBody {
    // Optional field to override the 5 minute rate limit
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReindexDriveResponseData {
    pub success: bool,
    pub timestamp_ms: u64,
    pub indexed_count: usize,
}

pub type ReindexDriveResponse<'a> = DriveResponse<'a, ReindexDriveResponseData>;