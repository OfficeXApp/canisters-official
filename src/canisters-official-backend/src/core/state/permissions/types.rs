// src/core/state/permissions/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;
use std::collections::HashSet;
use serde_diff::{SerdeDiff};

use crate::{core::{
    state::{
        directory::types::DriveFullFilePath, drives::types::{ExternalID, ExternalPayload}, tags::types::TagStringValue, teams::types::TeamID
    },
    types::UserID,
}, rest::directory::types::DirectoryResourceID};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DirectoryPermissionID(pub String);

impl fmt::Display for DirectoryPermissionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct PlaceholderPermissionGranteeID(pub String);

impl fmt::Display for PlaceholderPermissionGranteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectoryPermissionType {
    View,
    Upload,   // Can upload/edit/delete own files
    Edit,     // Can upload/edit peer files but not delete
    Delete,   // Can delete peer files
    Invite,   // Can invite other users with same or lower permissions
    Manage,   // Can do anything on this directory resource
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum PermissionGranteeID {
    Public,
    User(UserID),
    Team(TeamID),
    PlaceholderDirectoryPermissionGrantee(PlaceholderPermissionGranteeID),
}
impl fmt::Display for PermissionGranteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionGranteeID::Public => write!(f, "{}", PUBLIC_GRANTEE_ID),
            PermissionGranteeID::User(user_id) => write!(f, "{}", user_id),
            PermissionGranteeID::Team(team_id) => write!(f, "{}", team_id),
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(placeholder_id) => write!(f, "{}", placeholder_id),
        }
    }
}
pub const PUBLIC_GRANTEE_ID: &str = "PUBLIC";


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct DirectoryPermission {
    pub id: DirectoryPermissionID,
    pub resource_id: DirectoryResourceID,
    pub resource_path: DriveFullFilePath,
    pub granted_to: PermissionGranteeID,
    pub granted_by: UserID,
    pub permission_types: Vec<DirectoryPermissionType>,
    pub begin_date_ms: i64,     // -1: not yet active, 0: immediate, >0: unix ms
    pub expiry_date_ms: i64,    // -1: never expires, 0: expired, >0: unix ms
    pub inheritable: bool,      // Whether permission applies to sub-resources
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<PlaceholderPermissionGranteeID>,
    pub tags: Vec<TagStringValue>,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct SystemPermissionID(pub String);

impl fmt::Display for SystemPermissionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemPermissionType {
    Create,
    Update,
    Delete,
    View,
    Invite,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemTableEnum {
    Drives,
    Disks,
    Contacts,
    Teams,
    Api_Keys,
    Permissions,
    Webhooks,
    Tags
}

impl fmt::Display for SystemTableEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemTableEnum::Drives => write!(f, "DRIVES"),
            SystemTableEnum::Disks => write!(f, "DISKS"),
            SystemTableEnum::Contacts => write!(f, "CONTACTS"),
            SystemTableEnum::Teams => write!(f, "TEAMS"),
            SystemTableEnum::Api_Keys => write!(f, "API_KEYS"),
            SystemTableEnum::Permissions => write!(f, "PERMISSIONS"), // special enum, there is no record based permission permission, only a system wide permission that can edit all permissions
            SystemTableEnum::Webhooks => write!(f, "WEBHOOKS"),
            SystemTableEnum::Tags => write!(f, "TAGS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum SystemRecordIDEnum {
    Drive(String),        // DriveID_xxx
    Disk(String),         // DiskID_xxx
    User(String),      // UserID_xxx (for contacts)
    Team(String),         // TeamID_xxx
    ApiKey(String),       // ApiKeyID_xxx
    Permission(String),   // SystemPermissionID_xxx or DirectoryPermissionID_xxx
    Webhook(String),      // WebhookID_xxx
    Tag(String),          // TagID_xxx
    Unknown(String), // General catch
}

impl fmt::Display for SystemRecordIDEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemRecordIDEnum::Drive(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Disk(id) => write!(f, "{}", id),
            SystemRecordIDEnum::User(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Team(id) => write!(f, "{}", id),
            SystemRecordIDEnum::ApiKey(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Permission(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Webhook(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Tag(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Unknown(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum SystemResourceID {
    Table(SystemTableEnum),
    Record(SystemRecordIDEnum), // Stores the full ID like "DiskID_123"
}

impl fmt::Display for SystemResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemResourceID::Table(table) => write!(f, "Table_{}", table),
            SystemResourceID::Record(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct SystemPermission {
    pub id: SystemPermissionID,
    pub resource_id: SystemResourceID,
    pub granted_to: PermissionGranteeID,      // Reuse from directory permissions
    pub granted_by: UserID,
    pub permission_types: Vec<SystemPermissionType>,
    pub begin_date_ms: i64,     // -1: not yet active, 0: immediate, >0: unix ms
    pub expiry_date_ms: i64,    // -1: never expires, 0: expired, >0: unix ms
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<PlaceholderPermissionGranteeID>,
    pub tags: Vec<TagStringValue>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}


// TagStringValuePrefix definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TagStringValuePrefix(pub String);

impl fmt::Display for TagStringValuePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// The main metadata container
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct PermissionMetadata {
    pub metadata_type: PermissionMetadataTypeEnum, // Using existing enum but not assuming table connection
    pub content: PermissionMetadataContent,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionMetadataTypeEnum {
    Tags
}

impl fmt::Display for PermissionMetadataTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionMetadataTypeEnum::Tags => write!(f, "TAGS"),
        }
    }
}


// Define an enum for different types of metadata
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub enum PermissionMetadataContent {
    Tags(TagStringValuePrefix),
    // Future types can be added here without breaking changes
}
