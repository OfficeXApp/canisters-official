
// src/core/state/api_keys/types.rs
use serde_diff::{Diff, SerdeDiff};
use serde::{Deserialize, Serialize};
use crate::core::types::UserID;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ApiKeyID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ApiKeyValue(pub String);


// Implement Display for ApiKey
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey {{ id: {}, name: {}, user_id: {} }}", 
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
#[serde(rename_all = "PascalCase")]
pub enum AuthTypeEnum {
    Signature,
    ApiKey
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum AuthJsonDecoded {
    Signature(SignatureAuthProof),
    ApiKey(ApiKeyProof),
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