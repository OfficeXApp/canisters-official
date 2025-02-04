// src/rest/disks/types.rs

use serde::{Deserialize, Serialize};

use crate::core::state::disks::types::{DiskID, DiskItem};

#[derive(Debug, Clone, Serialize)]
pub enum DiskResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> DiskResponse<'a, T> {
    pub fn ok(data: &'a T) -> DiskResponse<T> {
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



pub type GetDiskResponse<'a> = DiskResponse<'a, DiskItem>;

pub type ListDisksResponse<'a> = DiskResponse<'a, Vec<DiskItem>>;


#[derive(Debug, Clone, Deserialize)]
pub struct CreateDiskRequest {
    pub title: String,
}

pub type CreateDiskResponse<'a> = DiskResponse<'a, DiskItem>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDiskRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateDiskResponse<'a> = DiskResponse<'a, DiskItem>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDiskRequest {
    pub id: DiskID,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedDiskData {
    pub id: DiskID,
    pub deleted: bool
}

pub type DeleteDiskResponse<'a> = DiskResponse<'a, DeletedDiskData>;


pub type ErrorResponse<'a> = DiskResponse<'a, ()>;