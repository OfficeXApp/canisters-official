// src/core/api/types.rs
use serde::{Deserialize, Serialize};

use crate::core::{state::drives::types::DriveID, types::PublicKeyICP};

#[derive(Debug)]
pub enum DirectoryError {
    FolderNotFound(String),
    // Add other error types as needed
}

#[derive(Debug)]
pub enum DirectoryIDError {
    InvalidPrefix,
    MalformedID,
    UnknownType,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthType {
    ApiKey,
    Signature,
}


#[derive(Serialize, Deserialize)]
pub struct AuthHeaderByApiKeySchema {
    pub auth_type: AuthType,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthHeaderBySignatureSchema {
    pub auth_type: AuthType,
    pub challenge: AuthSignatureChallenge,
    pub signature: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub struct AuthSignatureChallenge {
    pub timestamp_ms: u64,
    pub drive_canister_id: String,
    pub user_icp_public_key: PublicKeyICP, // client user
}