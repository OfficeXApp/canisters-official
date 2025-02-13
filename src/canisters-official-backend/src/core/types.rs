
// src/core/state/types.rs
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKeyBLS(pub String);

impl fmt::Display for PublicKeyBLS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ICPPrincipalString(pub PublicKeyBLS);

impl fmt::Display for ICPPrincipalString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserID(pub String);

impl fmt::Display for UserID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IDPrefix {
    File,
    Folder,
    Drive,
    ApiKey,
    Disk,
    Team,
    Invite,
    Webhook,
    User,
    DirectoryPermission,
    DirectoryShareDeferred,
    DirectoryActionOutcome
}

impl IDPrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            IDPrefix::File => "FileID_",
            IDPrefix::Folder => "FolderID_",
            IDPrefix::Drive => "DriveID_",
            IDPrefix::ApiKey => "ApiKeyID_",
            IDPrefix::Disk => "DiskID_",
            IDPrefix::Team => "TeamID_",
            IDPrefix::Invite => "InviteID_",
            IDPrefix::DirectoryPermission => "DirectoryPermissionID_",
            IDPrefix::DirectoryShareDeferred => "DirectoryShareDeferredID_",
            IDPrefix::Webhook => "WebhookID_",
            IDPrefix::User => "UserID_",
            IDPrefix::DirectoryActionOutcome => "DirectoryActionOutcomeID_",
        }
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthPrefixEnum {
    ApiKey,
    Signature,
}
impl AuthPrefixEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthPrefixEnum::ApiKey => "ApiKey_",
            AuthPrefixEnum::Signature => "Signature_",
        }
    }
}

#[derive(Debug)]
pub struct ParsedAuth {
    pub auth_type: AuthPrefixEnum,
    pub value: String,
}

pub fn parse_auth_header_value(auth_header_value: &str) -> Result<ParsedAuth, &'static str> {
    // First, check if it starts with "Bearer "
    let without_bearer = auth_header_value.strip_prefix("Bearer ")
        .ok_or("Authentication header must start with 'Bearer '")?;
    
    // Try to match against both prefix types
    if without_bearer.starts_with(AuthPrefixEnum::ApiKey.as_str()) {
        Ok(ParsedAuth {
            auth_type: AuthPrefixEnum::ApiKey,
            value: without_bearer[AuthPrefixEnum::ApiKey.as_str().len()..].to_string(),
        })
    } else if without_bearer.starts_with(AuthPrefixEnum::Signature.as_str()) {
        Ok(ParsedAuth {
            auth_type: AuthPrefixEnum::Signature,
            value: without_bearer[AuthPrefixEnum::Signature.as_str().len()..].to_string(),
        })
    } else {
        Err("Invalid authentication type prefix")
    }
}