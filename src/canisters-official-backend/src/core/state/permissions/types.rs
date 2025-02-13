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
pub struct PlaceholderDirectoryPermissionGranteeID(pub String);

impl fmt::Display for PlaceholderDirectoryPermissionGranteeID {
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
pub enum DirectoryGranteeID {
    Public,
    User(UserID),
    Team(TeamID),
    PlaceholderDirectoryPermissionGrantee(PlaceholderDirectoryPermissionGranteeID),
}
impl fmt::Display for DirectoryGranteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectoryGranteeID::Public => write!(f, "{}", PUBLIC_GRANTEE_ID),
            DirectoryGranteeID::User(user_id) => write!(f, "{}", user_id),
            DirectoryGranteeID::Team(team_id) => write!(f, "{}", team_id),
            DirectoryGranteeID::PlaceholderDirectoryPermissionGrantee(placeholder_id) => write!(f, "{}", placeholder_id),
        }
    }
}
pub const PUBLIC_GRANTEE_ID: &str = "PUBLIC";


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectoryGranteeType {
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
    pub grantee_type: DirectoryGranteeType,
    pub granted_to: DirectoryGranteeID,
    pub granted_by: UserID,
    pub permission_types: HashSet<DirectoryPermissionType>,
    pub begin_date_ms: i64,     // -1: not yet active, 0: immediate, >0: unix ms
    pub expiry_date_ms: i64,    // -1: never expires, 0: expired, >0: unix ms
    pub inheritable: bool,      // Whether permission applies to sub-resources
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<PlaceholderDirectoryPermissionGranteeID>,
}
