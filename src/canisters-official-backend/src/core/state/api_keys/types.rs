
// src/core/state/api_keys/types.rs
use serde_diff::{Diff, SerdeDiff};
use serde::{Deserialize, Serialize};
use crate::core::{api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}}, types::UserID};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ApiKeyID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ApiKeyValue(pub String);


// Implement Display for ApiKey
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API_KEY {{ id: {}, name: {}, user_id: {} }}", 
            self.id, self.name, self.user_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct ApiKey {
    pub id: ApiKeyID,
    pub value: ApiKeyValue,
    pub user_id: UserID,
    pub name: String,
    pub created_at: u64, 
    pub expires_at: i64, 
    pub is_revoked: bool,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}


impl ApiKey {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        // let is_owned = *user_id == self.user_id;
        // let table_permissions = check_system_permissions(
        //     SystemResourceID::Table(SystemTableEnum::Api_Keys),
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        // let permissions = check_system_permissions(
        //     resource_id,
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let has_edit_permissions = permissions.contains(&SystemPermissionType::Edit) || table_permissions.contains(&SystemPermissionType::Edit);

        // Filter tags
        redacted.tags = match is_owner {
            true => redacted.tags,
            false => redacted.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}


// Implement Display for ApiKeyID
impl fmt::Display for ApiKeyID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement Display for ApiKeyValue
impl fmt::Display for ApiKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


    
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthTypeEnum {
    Signature,
    Api_Key
}
impl fmt::Display for AuthTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthTypeEnum::Signature => write!(f, "SIGNATURE"),
            AuthTypeEnum::Api_Key => write!(f, "API_KEY"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum AuthJsonDecoded {
    Signature(SignatureAuthProof),
    Api_Key(ApiKeyProof),
}


#[derive(Deserialize, Serialize, Debug)]
pub struct ApiKeyProof {
    pub auth_type: AuthTypeEnum,
    pub value: ApiKeyValue,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SignatureAuthProof {
    pub auth_type: AuthTypeEnum,
    pub challenge: SignatureAuthChallenge,
    pub signature: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SignatureAuthChallenge {
    pub timestamp_ms: u64,
    pub drive_canister_id: String,
    pub self_auth_principal: Vec<u8>,
    pub canonical_principal: String,
}