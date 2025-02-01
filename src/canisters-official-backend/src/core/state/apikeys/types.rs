
// src/core/state/apikeys/types.rs

use serde::{Deserialize, Serialize};
use crate::core::types::UserID;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyValue(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct ApiKey {
    pub id: ApiKeyID,
    pub value: ApiKeyValue,
    pub user_id: UserID, 
    pub name: String, 
    pub created_at: u64, 
    pub expires_at: i64, 
    pub is_revoked: bool,
}
