// src/rest/apikeys/types.rs



use serde::{Deserialize, Serialize};

use crate::{core::{state::apikeys::types::{ApiKey, ApiKeyID}, types::UserID}, types::{UpsertCreateType, UpsertEditType}};

#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyHidden {
    pub id: ApiKeyID,
    pub user_id: UserID,
    pub name: String,
    pub created_at: u64,
    pub expires_at: i64,
    pub is_revoked: bool,
}

impl From<ApiKey> for ApiKeyHidden {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            user_id: key.user_id,
            name: key.name,
            created_at: key.created_at,
            expires_at: key.expires_at,
            is_revoked: key.is_revoked,
        }
    }
}


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
pub struct CreateApiKeyRequestBody {
    #[serde(rename = "__type")]
    pub type_field: UpsertCreateType,
    pub name: String,
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}
pub type CreateApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKey>;



#[derive(Debug, Clone, Deserialize)]
pub struct DeleteApiKeyRequestBody {
    pub id: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct DeletedApiKeyData {
    pub id: String,
    pub deleted: bool
}
pub type DeleteApiKeyResponse<'a> = ApiKeyResponse<'a, DeletedApiKeyData>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApiKeyRequestBody {
    #[serde(rename = "__type")]
    pub type_field: UpsertEditType,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_revoked: Option<bool>,
}
pub type UpdateApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKey>;


pub type ListApiKeysResponse<'a> = ApiKeyResponse<'a, Vec<ApiKeyHidden>>;

pub type GetApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKey>;

pub type ErrorResponse<'a> = ApiKeyResponse<'a, ()>;