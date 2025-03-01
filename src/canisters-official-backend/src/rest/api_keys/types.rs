// src/rest/api_keys/types.rs



use serde::{Deserialize, Serialize};

use crate::{core::{state::api_keys::types::{ApiKey, ApiKeyID}, types::UserID}, types::ValidationError};

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
    pub fn ok(data: &'a T) -> ApiKeyResponse<'a, T> {
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
#[serde(deny_unknown_fields)]
pub struct CreateApiKeyRequestBody {
    pub name: String,
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}
impl CreateApiKeyRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate name (up to 256 chars)
        if self.name.is_empty() {
            return Err(ValidationError {
                field: "name".to_string(),
                message: "Name cannot be empty".to_string(),
            });
        }
        if self.name.len() > 256 {
            return Err(ValidationError {
                field: "name".to_string(),
                message: "Name must be 256 characters or less".to_string(),
            });
        }

        // Validate user_id if provided (up to 256 chars)
        if let Some(user_id) = &self.user_id {
            if user_id.is_empty() {
                return Err(ValidationError {
                    field: "user_id".to_string(),
                    message: "User ID cannot be empty".to_string(),
                });
            }
            if user_id.len() > 256 {
                return Err(ValidationError {
                    field: "user_id".to_string(),
                    message: "User ID must be 256 characters or less".to_string(),
                });
            }
        }

        // Validate external_id if provided (up to 256 chars)
        if let Some(external_id) = &self.external_id {
            if external_id.len() > 256 {
                return Err(ValidationError {
                    field: "external_id".to_string(),
                    message: "External ID must be 256 characters or less".to_string(),
                });
            }
        }

        // Validate external_payload if provided (up to 8,192 chars)
        if let Some(external_payload) = &self.external_payload {
            if external_payload.len() > 8192 {
                return Err(ValidationError {
                    field: "external_payload".to_string(),
                    message: "External payload must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate expires_at if provided (must be a future timestamp)
        if let Some(expires_at) = self.expires_at {
            if expires_at != -1 && expires_at <= ic_cdk::api::time() as i64 {
                return Err(ValidationError {
                    field: "expires_at".to_string(),
                    message: "Expiration time must be in the future or -1 for never expires".to_string(),
                });
            }
        }

        Ok(())
    }
}
pub type CreateApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKey>;



#[derive(Debug, Clone, Deserialize)]
pub struct DeleteApiKeyRequestBody {
    pub id: String,
}
impl DeleteApiKeyRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty and up to 256 chars)
        if self.id.is_empty() {
            return Err(ValidationError {
                field: "id".to_string(),
                message: "ID cannot be empty".to_string(),
            });
        }
        if self.id.len() > 256 {
            return Err(ValidationError {
                field: "id".to_string(),
                message: "ID must be 256 characters or less".to_string(),
            });
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DeletedApiKeyData {
    pub id: String,
    pub deleted: bool
}
pub type DeleteApiKeyResponse<'a> = ApiKeyResponse<'a, DeletedApiKeyData>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApiKeyRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_revoked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}

impl UpdateApiKeyRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty and up to 256 chars)
        if self.id.is_empty() {
            return Err(ValidationError {
                field: "id".to_string(),
                message: "ID cannot be empty".to_string(),
            });
        }
        if self.id.len() > 256 {
            return Err(ValidationError {
                field: "id".to_string(),
                message: "ID must be 256 characters or less".to_string(),
            });
        }

        // Validate name if provided (up to 256 chars)
        if let Some(name) = &self.name {
            if name.is_empty() {
                return Err(ValidationError {
                    field: "name".to_string(),
                    message: "Name cannot be empty".to_string(),
                });
            }
            if name.len() > 256 {
                return Err(ValidationError {
                    field: "name".to_string(),
                    message: "Name must be 256 characters or less".to_string(),
                });
            }
        }

        // Validate external_id if provided (up to 256 chars)
        if let Some(external_id) = &self.external_id {
            if external_id.len() > 256 {
                return Err(ValidationError {
                    field: "external_id".to_string(),
                    message: "External ID must be 256 characters or less".to_string(),
                });
            }
        }

        // Validate external_payload if provided (up to 8,192 chars)
        if let Some(external_payload) = &self.external_payload {
            if external_payload.len() > 8192 {
                return Err(ValidationError {
                    field: "external_payload".to_string(),
                    message: "External payload must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate expires_at if provided (must be a future timestamp)
        if let Some(expires_at) = self.expires_at {
            if expires_at != -1 && expires_at <= ic_cdk::api::time() as i64 {
                return Err(ValidationError {
                    field: "expires_at".to_string(),
                    message: "Expiration time must be in the future or -1 for never expires".to_string(),
                });
            }
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertApiKeyRequestBody {
    Create(CreateApiKeyRequestBody),
    Update(UpdateApiKeyRequestBody),
}
impl UpsertApiKeyRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertApiKeyRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertApiKeyRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

pub type UpdateApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKey>;


pub type ListApiKeysResponse<'a> = ApiKeyResponse<'a, Vec<ApiKeyHidden>>;

pub type GetApiKeyResponse<'a> = ApiKeyResponse<'a, ApiKey>;

pub type ErrorResponse<'a> = ApiKeyResponse<'a, ()>;