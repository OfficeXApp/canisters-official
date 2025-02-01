// src/rest/apikeys/types.rs



use serde::{Deserialize, Serialize};

use crate::core::state::apikeys::types::ApiKeyItem;


#[derive(Debug, Clone, Serialize)]
pub enum ApiKeyResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> ApiKeyResponse<'a, T> {
    pub fn ok(data: &'a T) -> ApiKeyResponse<T> {
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn not_allowed() -> Self {
        Self::err(405, "Method not allowed".to_string())
    }

    fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize value")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteApiKeyRequest {
    pub id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedApiKeyData {
    pub deleted_id: u32,
}

pub type DeleteApiKeyResponse<'a> = ApiKeyResponse<'a, DeletedApiKeyData>;

pub type CreateApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKeyItem>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateApiKeyResponse<'a> = ApiKeyResponse<'a, ()>;

pub type ListApiKeysResponse<'a> = ApiKeyResponse<'a, Vec<ApiKeyItem>>;

pub type GetApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKeyItem>;

pub type ErrorResponse<'a> = ApiKeyResponse<'a, ()>;