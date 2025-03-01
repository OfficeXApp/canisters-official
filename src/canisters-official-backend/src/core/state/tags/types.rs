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
        drives::types::{DriveID, ExternalID, ExternalPayload},
        permissions::types::{DirectoryPermissionID, SystemPermissionID},
        team_invites::types::TeamInviteID,
        teams::types::TeamID,
        webhooks::types::WebhookID
    },
    types::{IDPrefix, UserID}
};

// TagID is the unique identifier for a tag
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TagID(pub String);

impl fmt::Display for TagID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TagStringValue is the actual text of the tag
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TagStringValue(pub String);

impl fmt::Display for TagStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// HexColorString represents a color in hex format
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct HexColorString(pub String);

impl fmt::Display for HexColorString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// The main Tag type that represents a tag definition
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Tag {
    pub id: TagID,
    pub value: TagStringValue,
    pub description: Option<String>,
    pub color: HexColorString,
    pub created_by: UserID,
    pub created_at: u64,
    pub last_updated_at: u64,
    pub resources: Vec<TagResourceID>,
    pub tags: Vec<TagStringValue>,  // Tags can be tagged too
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

// TagResourceID represents any resource that can be tagged
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
    Tag(TagID),
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
            TagResourceID::Tag(id) => write!(f, "{}", id),
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
            TagResourceID::Tag(id) => id.0.clone(),
        }
    }
}

// Request and response types for tag operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub value: String,
    pub description: Option<String>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTagRequest {
    pub id: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpsertTagRequest {
    Create(CreateTagRequest),
    Update(UpdateTagRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResourceRequest {
    pub tag_id: String,
    pub resource_id: String,
    pub add: bool,  // true to add, false to remove
}

#[derive(Debug, Clone, Serialize)]
pub struct TagOperationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub tag: Option<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTagsRequest {
    pub query: Option<String>,
    pub page_size: Option<usize>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListTagsResponse {
    pub items: Vec<Tag>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTagRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteTagResponse {
    pub success: bool,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTagResourcesRequest {
    pub tag_id: String,
    pub resource_type: Option<String>,
    pub page_size: Option<usize>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTagResourcesResponse {
    pub tag_id: String,
    pub resources: Vec<TagResourceID>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}