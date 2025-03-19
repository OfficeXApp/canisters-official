// src/core/state/permissions/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;
use std::collections::HashSet;
use serde_diff::{SerdeDiff};

use crate::{core::{
    api::permissions::system::check_system_permissions, state::{
        api_keys::types::ApiKeyID, directory::types::{DriveClippedFilePath, DriveFullFilePath}, disks::types::DiskID, drives::{state::state::OWNER_ID, types::{DriveID, ExternalID, ExternalPayload}}, groups::types::GroupID, labels::types::{redact_label, LabelID, LabelStringValue}, webhooks::types::WebhookID
    }, types::UserID
}, rest::{directory::types::DirectoryResourceID, permissions::types::{DirectoryPermissionFE, SystemPermissionFE}}};

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
// impl display
impl fmt::Display for DirectoryPermissionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectoryPermissionType::View => write!(f, "VIEW"),
            DirectoryPermissionType::Upload => write!(f, "UPLOAD"),
            DirectoryPermissionType::Edit => write!(f, "EDIT"),
            DirectoryPermissionType::Delete => write!(f, "DELETE"),
            DirectoryPermissionType::Invite => write!(f, "INVITE"),
            DirectoryPermissionType::Manage => write!(f, "MANAGE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum PermissionGranteeID {
    Public,
    User(UserID),
    Group(GroupID),
    PlaceholderDirectoryPermissionGrantee(PlaceholderPermissionGranteeID),
}
impl fmt::Display for PermissionGranteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionGranteeID::Public => write!(f, "{}", PUBLIC_GRANTEE_ID),
            PermissionGranteeID::User(user_id) => write!(f, "{}", user_id),
            PermissionGranteeID::Group(group_id) => write!(f, "{}", group_id),
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
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl DirectoryPermission {
    pub fn cast_fe(&self, user_id: &UserID) -> DirectoryPermissionFE {
        // Convert resource_id enum to string
        let resource_id = self.resource_id.to_string();
        
        // Convert granted_to enum to string
        let granted_to = match &self.granted_to {
            PermissionGranteeID::Public => "PUBLIC".to_string(),
            PermissionGranteeID::User(user_id) => user_id.to_string(),
            PermissionGranteeID::Group(group_id) => group_id.to_string(),
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(placeholder_id) => placeholder_id.to_string(),
        };
        
        // Convert from_placeholder_grantee to string if present
        let from_placeholder_grantee = self.from_placeholder_grantee.as_ref().map(|p| p.to_string());
        
        // Convert external_id to string if present
        let external_id = self.external_id.as_ref().map(|e| e.to_string());
        
        // Convert external_payload to string if present
        let external_payload = self.external_payload.as_ref().map(|e| e.to_string());
        
        // Get user's system permissions for this permission record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Permission(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Permissions),
            PermissionGranteeID::User(user_id.clone())
        );
        
        let permission_previews: Vec<SystemPermissionType> = record_permissions
            .into_iter()
            .chain(table_permissions)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // clip resource_path to only the disk & file or foldername
        // disk_id::path/to/folder/
        // disk_id::path/to/folder/file.txt
        // recostruct with .. in between
        // disk_id::../folder/
        // disk_id::../file.txt
        let resource_path = self.resource_path.clone();
        let path_parts = resource_path.0.split("/").collect::<Vec<&str>>();
        let mut clipped_path = String::new();
        if path_parts.len() > 1 {
            clipped_path.push_str(path_parts[0]);
            clipped_path.push_str("::");
            if path_parts.len() > 2 {
                clipped_path.push_str("..");
                clipped_path.push_str("/");
            }
            clipped_path.push_str(path_parts[path_parts.len()-1]);
        } else {
            clipped_path.push_str(&resource_path.0);
        }

        

        DirectoryPermissionFE {
            id: self.id.to_string(),
            resource_id,
            resource_path: DriveClippedFilePath(clipped_path),
            granted_to,
            granted_by: self.granted_by.to_string(),
            permission_types: self.permission_types.clone(),
            begin_date_ms: self.begin_date_ms,
            expiry_date_ms: self.expiry_date_ms,
            inheritable: self.inheritable,
            note: self.note.clone(),
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            from_placeholder_grantee,
            labels: self.labels.clone(),
            external_id,
            external_payload,
            permission_previews,
        }.redacted(user_id)
    }
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
    Edit,
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
    Groups,
    Api_Keys,
    Permissions,
    Webhooks,
    Labels,
    Inbox
}

impl fmt::Display for SystemTableEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemTableEnum::Drives => write!(f, "DRIVES"),
            SystemTableEnum::Disks => write!(f, "DISKS"),
            SystemTableEnum::Contacts => write!(f, "CONTACTS"),
            SystemTableEnum::Groups => write!(f, "GROUPS"),
            SystemTableEnum::Api_Keys => write!(f, "API_KEYS"),
            SystemTableEnum::Permissions => write!(f, "PERMISSIONS"), // special enum, there is no record based permission permission, only a system wide permission that can edit all permissions
            SystemTableEnum::Webhooks => write!(f, "WEBHOOKS"),
            SystemTableEnum::Labels => write!(f, "LABELS"),
            SystemTableEnum::Inbox => write!(f, "INBOX"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum SystemRecordIDEnum {
    Drive(String),        // DriveID_xxx
    Disk(String),         // DiskID_xxx
    User(String),      // UserID_xxx (for contacts)
    Group(String),         // GroupID_xxx
    ApiKey(String),       // ApiKeyID_xxx
    Permission(String),   // SystemPermissionID_xxx or DirectoryPermissionID_xxx
    Webhook(String),      // WebhookID_xxx
    Label(String),          // LabelID_xxx
    Unknown(String), // General catch
}

impl fmt::Display for SystemRecordIDEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemRecordIDEnum::Drive(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Disk(id) => write!(f, "{}", id),
            SystemRecordIDEnum::User(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Group(id) => write!(f, "{}", id),
            SystemRecordIDEnum::ApiKey(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Permission(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Webhook(id) => write!(f, "{}", id),
            SystemRecordIDEnum::Label(id) => write!(f, "{}", id),
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
            SystemResourceID::Table(table) => write!(f, "TABLE_{}", table),
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
    pub labels: Vec<LabelStringValue>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl SystemPermission {
    pub fn cast_fe(&self, user_id: &UserID) -> SystemPermissionFE {
        // Convert resource_id to string
        let resource_id = self.resource_id.to_string();
        
        // Convert granted_to to string
        let granted_to = match &self.granted_to {
            PermissionGranteeID::Public => "PUBLIC".to_string(),
            PermissionGranteeID::User(user_id) => user_id.to_string(),
            PermissionGranteeID::Group(group_id) => group_id.to_string(),
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(placeholder_id) => placeholder_id.to_string(),
        };
        
        // Get resource name based on the resource ID and its prefix
        let resource_name = match &self.resource_id {
            SystemResourceID::Record(record_id) => {
                match record_id {
                    SystemRecordIDEnum::User(id) if id.starts_with("UserID_") => {
                        crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                            .with(|contacts| {
                                contacts.borrow().get(&UserID(id.clone()))
                                    .map(|contact| contact.name.clone())
                            })
                    },
                    SystemRecordIDEnum::Group(id) if id.starts_with("GroupID_") => {
                        crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE
                            .with(|groups| {
                                groups.borrow().get(&GroupID(id.clone()))
                                    .map(|group| group.name.clone())
                            })
                    },
                    SystemRecordIDEnum::Drive(id) if id.starts_with("DriveID_") => {
                        crate::core::state::drives::state::state::DRIVES_BY_ID_HASHTABLE
                            .with(|drives| {
                                drives.borrow().get(&DriveID(id.clone()))
                                    .map(|drive| drive.name.clone())
                            })
                    },
                    SystemRecordIDEnum::Disk(id) if id.starts_with("DiskID_") => {
                        crate::core::state::disks::state::state::DISKS_BY_ID_HASHTABLE
                            .with(|disks| {
                                disks.borrow().get(&DiskID(id.clone()))
                                    .map(|disk| disk.name.clone())
                            })
                    },
                    SystemRecordIDEnum::ApiKey(id) if id.starts_with("ApiKeyID_") => {
                        crate::core::state::api_keys::state::state::APIKEYS_BY_ID_HASHTABLE
                            .with(|keys| {
                                keys.borrow().get(&ApiKeyID(id.clone()))
                                    .map(|key| key.name.clone())
                            })
                    },
                    SystemRecordIDEnum::Webhook(id) if id.starts_with("WebhookID_") => {
                        crate::core::state::webhooks::state::state::WEBHOOKS_BY_ID_HASHTABLE
                            .with(|webhooks| {
                                webhooks.borrow().get(&WebhookID(id.clone()))
                                    .map(|webhook| webhook.name.clone())
                            })
                    },
                    SystemRecordIDEnum::Label(id) if id.starts_with("LabelID_") => {
                        crate::core::state::labels::state::LABELS_BY_ID_HASHTABLE
                            .with(|labels| {
                                labels.borrow().get(&LabelID(id.clone()))
                                    .map(|label| label.value.0.clone())
                            })
                    },
                    SystemRecordIDEnum::Permission(id) if id.starts_with("SystemPermissionID_") => {
                        Some(format!("Permission {}", id))
                    },
                    _ => None,
                }
            },
            SystemResourceID::Table(table) => Some(format!("{:?} Table", table)),
        };
        
        // Get grantee name and avatar based on the grantee ID
        let (grantee_name, grantee_avatar) = match &self.granted_to {
            PermissionGranteeID::User(id) => {
                crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                    .with(|contacts| {
                        contacts.borrow().get(id)
                            .map(|contact| (contact.name.clone(), contact.avatar.clone()))
                            .unwrap_or((String::new(), None))
                    })
            },
            PermissionGranteeID::Group(id) => {
                crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE
                    .with(|groups| {
                        groups.borrow().get(id)
                            .map(|group| (group.name.clone(), group.avatar.clone()))
                            .unwrap_or((String::new(), None))
                    })
            },
            PermissionGranteeID::Public => {
                ("PUBLIC".to_string(), None)
            },
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(id) => {
                (format!("PLACEHOLDER: {}", id), None)
            },
        };
        
        // Get granter name based on the granter ID
        let granter_name = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
            .with(|contacts| {
                contacts.borrow().get(&self.granted_by)
                    .map(|contact| contact.name.clone())
            });
        
        // Convert from_placeholder_grantee to string if present
        let from_placeholder_grantee = self.from_placeholder_grantee.as_ref().map(|p| p.to_string());
        
        // Convert external_id to string if present
        let external_id = self.external_id.as_ref().map(|e| e.to_string());
        
        // Convert external_payload to string if present
        let external_payload = self.external_payload.as_ref().map(|e| e.to_string());
        
        // Get user's system permissions for this permission record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Permission(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        
        // Get permissions for the system permissions table
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Permissions),
            PermissionGranteeID::User(user_id.clone())
        );
        
        // Combine and deduplicate permissions
        let permission_previews: Vec<SystemPermissionType> = record_permissions
            .into_iter()
            .chain(table_permissions)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        SystemPermissionFE {
            id: self.id.to_string(),
            resource_id,
            granted_to,
            granted_by: self.granted_by.to_string(),
            permission_types: self.permission_types.clone(),
            begin_date_ms: self.begin_date_ms,
            expiry_date_ms: self.expiry_date_ms,
            note: self.note.clone(),
            created_at: self.created_at,
            last_modified_at: self.last_modified_at,
            from_placeholder_grantee,
            labels: self.labels.clone(),
            metadata: self.metadata.clone(),
            external_id,
            external_payload,
            resource_name,
            grantee_name: Some(grantee_name),
            grantee_avatar,
            granter_name,
            permission_previews,
        }.redacted(user_id)
    }
}

// LabelStringValuePrefix definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct LabelStringValuePrefix(pub String);

impl fmt::Display for LabelStringValuePrefix {
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
    Labels
}

impl fmt::Display for PermissionMetadataTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionMetadataTypeEnum::Labels => write!(f, "LABELS"),
        }
    }
}


// Define an enum for different types of metadata
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub enum PermissionMetadataContent {
    Labels(LabelStringValuePrefix),
    // Future types can be added here without breaking changes
}
