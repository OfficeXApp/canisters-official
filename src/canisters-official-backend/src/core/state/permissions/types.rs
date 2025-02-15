// src/core/state/permissions/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;
use std::collections::HashSet;

use crate::{core::{
    state::{
        directory::types::DriveFullFilePath,
        teams::types::TeamID,
    },
    types::UserID,
}, rest::directory::types::DirectoryResourceID};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DirectoryPermissionID(pub String);

impl fmt::Display for DirectoryPermissionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaceholderPermissionGranteeID(pub String);

impl fmt::Display for PlaceholderPermissionGranteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectoryPermissionType {
    View,
    Upload,   // Can upload/edit/delete own files
    Edit,     // Can upload/edit peer files but not delete
    Delete,   // Can delete peer files
    Webhooks, // Can set webhooks
    Invite,   // Can invite other users with same or lower permissions
    Manage,   // Can do anything on this directory resource
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionGranteeType {
    Public,
    User,
    Team,
    PlaceholderDirectoryPermissionGrantee,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryPermission {
    pub id: DirectoryPermissionID,
    pub resource_id: DirectoryResourceID,
    pub resource_path: DriveFullFilePath,
    pub grantee_type: PermissionGranteeType,
    pub granted_to: PermissionGranteeID,
    pub granted_by: UserID,
    pub permission_types: HashSet<DirectoryPermissionType>,
    pub begin_date_ms: i64,     // -1: not yet active, 0: immediate, >0: unix ms
    pub expiry_date_ms: i64,    // -1: never expires, 0: expired, >0: unix ms
    pub inheritable: bool,      // Whether permission applies to sub-resources
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<PlaceholderPermissionGranteeID>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemPermissionID(pub String);

impl fmt::Display for SystemPermissionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemPermissionType {
    Create,
    Update,
    Delete,
    View,
    Invite,
    Manage,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemTableEnum {
    Drives,
    Disks,
    Contacts,
    Teams,
    ApiKeys,
    Permissions,
}

impl fmt::Display for SystemTableEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemTableEnum::Drives => write!(f, "drives"),
            SystemTableEnum::Disks => write!(f, "disks"),
            SystemTableEnum::Contacts => write!(f, "contacts"),
            SystemTableEnum::Teams => write!(f, "teams"),
            SystemTableEnum::ApiKeys => write!(f, "api_keys"),
            SystemTableEnum::Permissions => write!(f, "permissions"), // special enum, there is no record based permission permission, only a system wide permission that can edit all permissions
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemResourceID {
    Table(SystemTableEnum),
    Record(String), // Stores the full ID like "DiskID_123"
}

impl fmt::Display for SystemResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemResourceID::Table(table) => write!(f, "Table_{}", table),
            SystemResourceID::Record(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPermission {
    pub id: SystemPermissionID,
    pub resource_id: SystemResourceID,
    pub grantee_type: PermissionGranteeType,  // Reuse from directory permissions
    pub granted_to: PermissionGranteeID,      // Reuse from directory permissions
    pub granted_by: UserID,
    pub permission_types: HashSet<SystemPermissionType>,
    pub begin_date_ms: i64,     // -1: not yet active, 0: immediate, >0: unix ms
    pub expiry_date_ms: i64,    // -1: never expires, 0: expired, >0: unix ms
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<PlaceholderPermissionGranteeID>,
}