// src/rest/api_keys/types.rs

use serde::{Deserialize, Serialize};
use crate::{core::{api::permissions::system::check_system_permissions, state::{api_keys::types::{ApiKey, ApiKeyID, ApiKeyValue}, drives::state::state::OWNER_ID, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, labels::{state::validate_uuid4_string_with_prefix, types::{redact_label, LabelStringValue}}}, types::{ClientSuggestedUUID, IDPrefix, UserID}}, rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_unclaimed_uuid, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyFE {
    #[serde(flatten)] 
    pub apiKey: ApiKey,
    pub user_name: Option<String>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl ApiKeyFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {
            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.apiKey.private_note = None;
            }
            // apiKey value is visible to its owner
            if (user_id != &redacted.apiKey.user_id) {
                redacted.apiKey.value = ApiKeyValue("".to_string());
            }
        }
        // Filter labels
        redacted.apiKey.labels = match is_owner {
            true => redacted.apiKey.labels,
            false => redacted.apiKey.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };

        
        redacted
    }
}




#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateApiKeyRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub name: String,
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begins_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}
impl CreateApiKeyRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::ApiKey)?;
        }

        // Validate name (up to 256 chars)
        validate_id_string(&self.name, "name")?;

        // Validate user_id if provided (must be a valid ICP principal with prefix)
        if let Some(user_id) = &self.user_id {
            validate_user_id(user_id)?;
        }

        // validate name
        validate_description(&self.name, "name")?;

        // validate private note
        if let Some(private_note) = &self.private_note {
            validate_description(private_note, "private_note")?;
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
            if expires_at != -1 && expires_at <= (ic_cdk::api::time() / 1_000_000) as i64 {
                return Err(ValidationError {
                    field: "expires_at".to_string(),
                    message: "Expiration time must be in the future or -1 for never expires".to_string(),
                });
            }
        }

        Ok(())
    }
}
pub type CreateApiKeyResponse<'a> = ApiResponse<'a, ApiKeyFE>;



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
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begins_at: Option<u64>,
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

        // validate private note
        if let Some(private_note) = &self.private_note {
            validate_description(private_note, "private_note")?;
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
            if expires_at != -1 && expires_at <= (ic_cdk::api::time() / 1_000_000) as i64 {
                return Err(ValidationError {
                    field: "expires_at".to_string(),
                    message: "Expiration time must be in the future or -1 for never expires".to_string(),
                });
            }
        }

        Ok(())
    }
}



pub type UpdateApiKeyResponse<'a> = ApiResponse<'a, ApiKeyFE>;
pub type ListApiKeysResponse<'a> = ApiResponse<'a, Vec<ApiKeyFE>>;
pub type GetApiKeyResponse<'a> = ApiResponse<'a, ApiKeyFE>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;