// src/rest/disks/types.rs

use serde::{Deserialize, Serialize};

use crate::{
    core::state::disks::types::{Disk, DiskID, DiskTypeEnum},
    rest::webhooks::types::SortDirection,
};

#[derive(Debug, Clone, Deserialize)]
pub struct ListDisksRequestBody {
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
pub struct ListDisksResponseData {
    pub items: Vec<Disk>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertDiskRequestBody {
    Create(CreateDiskRequestBody),
    Update(UpdateDiskRequestBody),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDiskRequestBody {
    pub name: String,
    pub disk_type: DiskTypeEnum,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub auth_json: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDiskRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDiskRequest {
    pub id: DiskID,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedDiskData {
    pub id: DiskID,
    pub deleted: bool,
}

pub type GetDiskResponse<'a> = DiskResponse<'a, Disk>;
pub type DeleteDiskResponse<'a> = DiskResponse<'a, DeletedDiskData>;
pub type ErrorResponse<'a> = DiskResponse<'a, ()>;
pub type ListDisksResponse<'a> = DiskResponse<'a, ListDisksResponseData>;
pub type CreateDiskResponse<'a> = DiskResponse<'a, Disk>;
pub type UpdateDiskResponse<'a> = DiskResponse<'a, Disk>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskResponse<'a, T>
where
    T: Serialize,
{
    Ok { data: &'a T },
    Err { code: u16, message: String },
}

impl<'a, T> DiskResponse<'a, T>
where
    T: Serialize,
{
    pub fn ok(data: &'a T) -> Self {
        DiskResponse::Ok { data }
    }

    pub fn err(code: u16, message: String) -> Self {
        DiskResponse::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_else(|_| 
            serde_json::to_vec(&DiskResponse::Err::<()> {
                code: 500,
                message: "Failed to serialize response".to_string(),
            }).unwrap_or_default()
        )
    }
}

impl<'a> DiskResponse<'a, ()> {
    pub fn not_found() -> Self {
        DiskResponse::Err {
            code: 404,
            message: "Not found".to_string(),
        }
    }
}