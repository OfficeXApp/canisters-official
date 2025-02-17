// src/rest/drives/types.rs

use serde::{Deserialize, Serialize};
use crate::core::state::drives::types::{DriveID, Drive};
use crate::core::types::PublicKeyICP;
use crate::rest::webhooks::types::SortDirection;

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