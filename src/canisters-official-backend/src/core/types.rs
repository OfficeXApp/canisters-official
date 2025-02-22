
// src/core/state/types.rs
use std::fmt;
use serde::{Deserialize, Serialize};

use crate::debug_log;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKeyICP(pub String);

impl fmt::Display for PublicKeyICP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ICPPrincipalString(pub PublicKeyICP);

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
    SystemPermission,
    DirectoryPermission,
    PlaceholderPermissionGrantee,
    DirectoryActionOutcome,
    PlaceholderTeamInviteeID
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
            IDPrefix::SystemPermission => "SystemPermissionID_",
            IDPrefix::DirectoryPermission => "DirectoryPermissionID_",
            IDPrefix::PlaceholderPermissionGrantee => "PlaceholderPermissionGranteeID_",
            IDPrefix::Webhook => "WebhookID_",
            IDPrefix::User => "UserID_",
            IDPrefix::DirectoryActionOutcome => "DirectoryActionOutcomeID_",
            IDPrefix::PlaceholderTeamInviteeID => "PlaceholderTeamInviteeID_",
        }
    }
}

