// src/core/state/tags/types.rs

use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;

use crate::core::{
    api::permissions::system::{check_system_permissions, check_system_resource_permissions_tags}, state::{
        api_keys::types::ApiKeyID,
        contacts::types::Contact,
        directory::types::{FileID, FolderID},
        disks::types::DiskID,
        drives::{state::state::OWNER_ID, types::{DriveID, ExternalID, ExternalPayload}},
        permissions::types::{DirectoryPermissionID, PermissionGranteeID, SystemPermissionID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum},
        team_invites::types::TeamInviteID,
        teams::types::TeamID,
        webhooks::types::WebhookID
    }, types::{IDPrefix, UserID}
};

use super::state::TAGS_BY_VALUE_HASHTABLE;

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
// We also dont redact tags here, for convinience. if we find this is a security issue, we can redact tags here too
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Tag {
    pub id: TagID,
    pub value: TagStringValue,
    pub description: Option<String>,
    pub color: HexColorString,
    pub created_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub created_at: u64,
    pub last_updated_at: u64,
    pub resources: Vec<TagResourceID>,
    pub tags: Vec<TagStringValue>,  // Tags can be tagged too
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Tag {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        // let table_permissions = check_system_permissions(
        //     SystemResourceID::Table(SystemTableEnum::Tags),
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        // let permissions = check_system_permissions(
        //     resource_id,
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let has_edit_permissions = permissions.contains(&SystemPermissionType::Update) || table_permissions.contains(&SystemPermissionType::Update);

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


// TagResourceID represents any resource that can be tagged
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum TagResourceID {
    ApiKey(ApiKeyID),
    Contact(UserID),
    File(FileID),
    Folder(FolderID),
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

pub fn redact_tag(tag_value: TagStringValue, user_id: UserID) -> Option<TagStringValue> {
    // Get the tag ID from the value
    let tag_id = TAGS_BY_VALUE_HASHTABLE.with(|store| {
        store.borrow().get(&tag_value).cloned()
    });
    
    if let Some(tag_id) = tag_id {
        // Check if the user is the owner
        let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow());
        
        if is_owner {
            // Owner sees everything, no redaction needed
            return Some(tag_value);
        }
        
        // Check permissions for this specific tag
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Tag(tag_id.to_string()));
        let permissions = check_system_resource_permissions_tags(
            &resource_id,
            &PermissionGranteeID::User(user_id.clone()),
            &tag_value.to_string()
        );
        
        // Check permissions for the Tags table
        let table_permissions = check_system_resource_permissions_tags(
            &SystemResourceID::Table(SystemTableEnum::Tags),
            &PermissionGranteeID::User(user_id.clone()),
            &tag_value.to_string()
        );
        
        // If the user has View permission either at the table level or for this specific tag
        if permissions.contains(&SystemPermissionType::View) || table_permissions.contains(&SystemPermissionType::View) {
            return Some(tag_value);
        }

        // Check if there are any permissions with tag prefixes that would allow viewing
        // (This is already handled by check_system_resource_permissions_tags)
        
        // If we get here, the user doesn't have permission to see this tag
        return None;
    }
    
    // Tag not found, so we can't provide it
    None
}