
// src/core/state/api_keys/types.rs

use serde::{Deserialize, Serialize};
use crate::core::types::UserID;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiKeyValue(pub String);


// Implement Display for ApiKey
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey {{ id: {}, name: {}, user_id: {} }}", 
            self.id, self.name, self.user_id)
    }
}

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