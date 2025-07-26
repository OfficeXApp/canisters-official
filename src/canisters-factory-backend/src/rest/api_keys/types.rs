// src/rest/api_keys/types.rs

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::{core::{state::api_keys::types::{ApiKey, ApiKeyID, ApiKeyValue}, types::{IDPrefix, PublicKeyICP, UserID}}, rest::types::{validate_external_id, validate_external_payload, validate_id_string, validate_user_id, ApiResponse, ValidationError}};
use crate::core::state::giftcards_spawnorg::types::DriveID;
use crate::core::state::giftcards_spawnorg::types::DriveRESTUrlEndpoint;
use crate::core::state::giftcards_spawnorg::types::FactorySpawnHistoryRecord;
use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrgID;
use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrg;



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
pub type ListApiKeysResponse<'a> = ApiResponse<'a, Vec<ApiKey>>;
pub type GetApiKeyResponse<'a> = ApiResponse<'a, ApiKey>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateSnapshot {
    // System info
    pub canister_id: PublicKeyICP,
    pub version: String,
    pub owner_id: UserID,
    pub endpoint_url: DriveRESTUrlEndpoint,
    
    // API keys state
    pub apikeys_by_value: HashMap<ApiKeyValue, ApiKeyID>,
    pub apikeys_by_id: HashMap<ApiKeyID, ApiKey>,
    pub users_apikeys: HashMap<UserID, Vec<ApiKeyID>>,
    pub apikeys_history: Vec<ApiKeyID>,
    
    // GiftcardSpawnOrg state
    pub deployments_by_giftcard_id: HashMap<GiftcardSpawnOrgID, FactorySpawnHistoryRecord>,
    pub historical_giftcards: Vec<GiftcardSpawnOrgID>,
    pub drive_to_giftcard_hashtable: HashMap<DriveID, GiftcardSpawnOrgID>,
    pub user_to_giftcards_hashtable: HashMap<UserID, Vec<GiftcardSpawnOrgID>>,
    pub giftcard_by_id: HashMap<GiftcardSpawnOrgID, GiftcardSpawnOrg>,
    
    // Timestamp
    pub timestamp_ns: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SnapshotResponse {
    pub status: String,
    pub data: StateSnapshot,
    pub timestamp: u64,
}

impl SnapshotResponse {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}