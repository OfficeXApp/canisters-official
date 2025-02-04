// src/rest/drives/types.rs

use serde::{Deserialize, Serialize};

use crate::core::state::drives::types::{DriveID, Drive};

#[derive(Debug, Clone, Serialize)]
pub enum DriveResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> DriveResponse<'a, T> {
    pub fn ok(data: &'a T) -> DriveResponse<T> {
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



pub type GetDriveResponse<'a> = DriveResponse<'a, Drive>;

pub type ListDrivesResponse<'a> = DriveResponse<'a, Vec<Drive>>;


#[derive(Debug, Clone, Deserialize)]
pub struct CreateDriveRequest {
    pub title: String,
}

pub type CreateDriveResponse<'a> = DriveResponse<'a, Drive>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDriveRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

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