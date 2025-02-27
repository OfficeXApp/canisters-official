// src/core/state/tags/types.rs

use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;

use crate::core::{
    state::{
        api_keys::types::ApiKeyID,
        contacts::types::Contact,
        directory::types::{FileUUID, FolderUUID},
        disks::types::DiskID,
        drives::types::DriveID,
        permissions::types::{DirectoryPermissionID, SystemPermissionID},
        team_invites::types::TeamInviteID,
        teams::types::TeamID,
        webhooks::types::WebhookID
    },
    types::UserID
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TagStringValue(pub String);

impl fmt::Display for TagStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum TagResourceID {
    ApiKey(ApiKeyID),
    Contact(UserID),
    File(FileUUID),
    Folder(FolderUUID),
    Disk(DiskID),
    Drive(DriveID),
    DirectoryPermission(DirectoryPermissionID),
    SystemPermission(SystemPermissionID),
    TeamInvite(TeamInviteID),
    Team(TeamID),
    Webhook(WebhookID),
}

impl fmt::Display for TagResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagResourceID::ApiKey(id) => write!(f, "{}", id),
            TagResourceID::Contact(id) => write!(f, "{}", id),
            TagResourceID::File(id) => write!(f, "{}", id),
            TagResourceID::Folder(id) => write!(f, "{}", id),
            TagResourceID::Disk(id) => write!(f, "{}", id),
            TagResourceID::Drive(id) => write!(f, "{}", id),
            TagResourceID::DirectoryPermission(id) => write!(f, "{}", id),
            TagResourceID::SystemPermission(id) => write!(f, "{}", id),
            TagResourceID::TeamInvite(id) => write!(f, "{}", id),
            TagResourceID::Team(id) => write!(f, "{}", id),
            TagResourceID::Webhook(id) => write!(f, "{}", id),
        }
    }
}

impl TagResourceID {
    pub fn get_id_string(&self) -> String {
        match self {
            TagResourceID::ApiKey(id) => id.0.clone(),
            TagResourceID::Contact(id) => id.0.clone(),
            TagResourceID::File(id) => id.0.clone(),
            TagResourceID::Folder(id) => id.0.clone(),
            TagResourceID::Disk(id) => id.0.clone(),
            TagResourceID::Drive(id) => id.0.clone(),
            TagResourceID::DirectoryPermission(id) => id.0.clone(),
            TagResourceID::SystemPermission(id) => id.0.clone(),
            TagResourceID::TeamInvite(id) => id.0.clone(),
            TagResourceID::Team(id) => id.0.clone(),
            TagResourceID::Webhook(id) => id.0.clone(),
        }
    }
}

// Request and response types for tag operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagOperationRequest {
    pub resource_id: String,
    pub tag: String,
    pub upsert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagOperationResponse {
    pub success: bool,
    pub message: Option<String>,
}