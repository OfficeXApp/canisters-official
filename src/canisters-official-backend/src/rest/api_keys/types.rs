// src/rest/api_keys/types.rs

use serde::{Deserialize, Serialize};
use crate::{core::{state::api_keys::types::{ApiKey, ApiKeyID}, types::{IDPrefix, UserID}}, rest::types::{validate_external_id, validate_external_payload, validate_id_string, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}};

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




#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateApiKeyRequestBody {
    pub action: UpsertActionTypeEnum,
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
        validate_id_string(&self.name, "name")?;

        // Validate user_id if provided (must be a valid ICP principal with prefix)
        if let Some(user_id) = &self.user_id {
            validate_user_id(user_id)?;
        }

        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }

        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
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
pub type CreateApiKeyResponse<'a> = ApiResponse<'a, ApiKey>;



#[derive(Debug, Clone, Deserialize)]
pub struct DeleteApiKeyRequestBody {
    pub id: String,
}
impl DeleteApiKeyRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::ApiKey.as_str();
        if !self.id.starts_with(api_key_prefix) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("API Key ID must start with '{}'", api_key_prefix),
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
pub type DeleteApiKeyResponse<'a> = ApiResponse<'a, DeletedApiKeyData>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateApiKeyRequestBody {
    pub action: UpsertActionTypeEnum,
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
        // Validate id (must not be empty, up to 256 chars, and start with ApiKeyID_ prefix)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::ApiKey.as_str();
        if !self.id.starts_with(api_key_prefix) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("API Key ID must start with '{}'", api_key_prefix),
            });
        }

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_id_string(name, "name")?;
        }

        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }

        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
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

pub type UpdateApiKeyResponse<'a> = ApiResponse<'a, ApiKey>;
pub type ListApiKeysResponse<'a> = ApiResponse<'a, Vec<ApiKeyHidden>>;
pub type GetApiKeyResponse<'a> = ApiResponse<'a, ApiKey>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;