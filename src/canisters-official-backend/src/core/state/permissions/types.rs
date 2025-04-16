use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
// src/core/state/permissions/types.rs
use serde::{Serialize, Deserialize};
use std::{borrow::Cow, fmt};
use std::collections::HashSet;
use serde_diff::{SerdeDiff};

use crate::{core::{
    api::permissions::system::check_system_permissions, state::{
        api_keys::types::ApiKeyID, directory::types::{DriveClippedFilePath, DriveFullFilePath}, disks::types::DiskID, drives::{state::state::OWNER_ID, types::{DriveID, ExternalID, ExternalPayload}}, groups::types::GroupID, labels::types::{redact_label, LabelID, LabelStringValue}, webhooks::types::WebhookID
    }, types::{IDPrefix, UserID}
}, rest::{directory::types::DirectoryResourceID, permissions::types::{DirectoryPermissionFE, SystemPermissionFE}}};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct DirectoryPermissionID(pub String);

impl Storable for DirectoryPermissionID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DirectoryPermissionID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DirectoryPermissionID")
    }
}

impl fmt::Display for DirectoryPermissionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct PlaceholderPermissionGranteeID(pub String);

impl fmt::Display for PlaceholderPermissionGranteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub enum PermissionGranteeID {
    Public,
    User(UserID),
    Group(GroupID),
    PlaceholderDirectoryPermissionGrantee(PlaceholderPermissionGranteeID),
}
impl Storable for PermissionGranteeID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize PermissionGranteeID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize PermissionGranteeID")
    }
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


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd, PartialEq, Eq)]
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
    pub redeem_code: Option<String>,
    pub from_placeholder_grantee: Option<PlaceholderPermissionGranteeID>,
    pub metadata: Option<PermissionMetadata>,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Storable for DirectoryPermission {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DirectoryPermission");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DirectoryPermission")
    }
}

impl DirectoryPermission {
    pub fn cast_fe(&self, user_id: &UserID) -> DirectoryPermissionFE {
        // Convert resource_id enum to string
        
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
                ("Awaiting Anon".to_string(), None)
            },
        };
        
        // Get granter name based on the granter ID
        let granter_name = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
            .with(|contacts| {
                contacts.borrow().get(&self.granted_by)
                    .map(|contact| contact.name.clone())
            });
        
        DirectoryPermissionFE {
            id: self.id.clone().to_string(),
            resource_id: self.resource_id.clone().to_string(),
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
            redeem_code: self.redeem_code.clone(),
            metadata: self.metadata.clone(),
            permission_previews,
            resource_name: None,
            grantee_name: Some(grantee_name),
            grantee_avatar,
            granter_name,
        }.redacted(user_id)
    }
}




#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct SystemPermissionID(pub String);

impl Storable for SystemPermissionID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize SystemPermissionID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize SystemPermissionID")
    }
}

impl fmt::Display for SystemPermissionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemPermissionType {
    Create,
    Edit,
    Delete,
    View,
    Invite,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemTableEnum {
    Drives,
    Disks,
    Contacts,
    Groups,
    ApiKeys,
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
            SystemTableEnum::ApiKeys => write!(f, "API_KEYS"),
            SystemTableEnum::Permissions => write!(f, "PERMISSIONS"), // special enum, there is no record based permission permission, only a system wide permission that can edit all permissions
            SystemTableEnum::Webhooks => write!(f, "WEBHOOKS"),
            SystemTableEnum::Labels => write!(f, "LABELS"),
            SystemTableEnum::Inbox => write!(f, "INBOX"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
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

impl Storable for SystemRecordIDEnum {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize SystemRecordIDEnum");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize SystemRecordIDEnum")
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub enum SystemResourceID {
    Table(SystemTableEnum),
    Record(SystemRecordIDEnum), // Stores the full ID like "DiskID_123"
}
impl Storable for SystemResourceID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize SystemResourceID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize SystemResourceID")
    }
}

impl fmt::Display for SystemResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemResourceID::Table(table) => write!(f, "TABLE_{}", table),
            SystemResourceID::Record(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType)]
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
    pub redeem_code: Option<String>,
    pub from_placeholder_grantee: Option<PlaceholderPermissionGranteeID>,
    pub labels: Vec<LabelStringValue>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Storable for SystemPermission {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize SystemPermission");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize SystemPermission")
    }
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
                ("Awaiting Anon".to_string(), None)
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
            redeem_code: self.redeem_code.clone(),
            grantee_name: Some(grantee_name),
            grantee_avatar,
            granter_name,
            permission_previews,
        }.redacted(user_id)
    }
}

// LabelStringValuePrefix definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct LabelStringValuePrefix(pub String);

impl fmt::Display for LabelStringValuePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// The main metadata container
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct PermissionMetadata {
    pub metadata_type: PermissionMetadataTypeEnum, // Using existing enum but not assuming table connection
    pub content: PermissionMetadataContent,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionMetadataTypeEnum {
    Labels,
    DirectoryPassword
}

impl fmt::Display for PermissionMetadataTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionMetadataTypeEnum::Labels => write!(f, "LABELS"),
            PermissionMetadataTypeEnum::DirectoryPassword => write!(f, "DIRECTORY_PASSWORD"),
        }
    }
}


// Define an enum for different types of metadata
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub enum PermissionMetadataContent {
    Labels(LabelStringValuePrefix),
    DirectoryPassword(String),
    // Future types can be added here without breaking changes
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, SerdeDiff)]
pub struct DirectoryPermissionIDList {
    pub permissions: Vec<DirectoryPermissionID>,
}

impl Default for DirectoryPermissionIDList {
    fn default() -> Self {
        Self { permissions: Vec::new() }
    }
}

impl DirectoryPermissionIDList {
    pub fn new() -> Self {
        Self { permissions: Vec::new() }
    }
    
    pub fn with_permission(permission_id: DirectoryPermissionID) -> Self {
        Self { permissions: vec![permission_id] }
    }
    
    pub fn add(&mut self, permission_id: DirectoryPermissionID) {
        self.permissions.push(permission_id);
    }
    
    pub fn remove(&mut self, permission_id: &DirectoryPermissionID) -> bool {
        if let Some(pos) = self.permissions.iter().position(|k| k == permission_id) {
            self.permissions.remove(pos);
            true
        } else {
            false
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &DirectoryPermissionID> {
        self.permissions.iter()
    }
    
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

// From<Vec<DirectoryPermissionID>> for DirectoryPermissionIDList
impl From<Vec<DirectoryPermissionID>> for DirectoryPermissionIDList {
    fn from(permissions: Vec<DirectoryPermissionID>) -> Self {
        Self { permissions }
    }
}

// From<DirectoryPermissionIDList> for Vec<DirectoryPermissionID>
impl From<DirectoryPermissionIDList> for Vec<DirectoryPermissionID> {
    fn from(list: DirectoryPermissionIDList) -> Self {
        list.permissions
    }
}


impl Storable for DirectoryPermissionIDList {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 1024 * 4, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// Same for SystemPermissionIDList
#[derive(Clone, Debug, CandidType, Deserialize, Serialize, SerdeDiff)]
pub struct SystemPermissionIDList {
    pub permissions: Vec<SystemPermissionID>,
}

impl SystemPermissionIDList {
    pub fn new() -> Self {
        Self { permissions: Vec::new() }
    }
    
    pub fn with_permission(permission_id: SystemPermissionID) -> Self {
        Self { permissions: vec![permission_id] }
    }
    
    pub fn add(&mut self, permission_id: SystemPermissionID) {
        self.permissions.push(permission_id);
    }
    
    pub fn remove(&mut self, permission_id: &SystemPermissionID) -> bool {
        if let Some(pos) = self.permissions.iter().position(|k| k == permission_id) {
            self.permissions.remove(pos);
            true
        } else {
            false
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &SystemPermissionID> {
        self.permissions.iter()
    }
    
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

// From<Vec<SystemPermissionID>> for SystemPermissionIDList
impl From<Vec<SystemPermissionID>> for SystemPermissionIDList {
    fn from(permissions: Vec<SystemPermissionID>) -> Self {
        Self { permissions }
    }
}

// From<SystemPermissionIDList> for Vec<SystemPermissionID>
impl From<SystemPermissionIDList> for Vec<SystemPermissionID> {
    fn from(list: SystemPermissionIDList) -> Self {
        list.permissions
    }
}

// default empty vec
impl Default for SystemPermissionIDList {
    fn default() -> Self {
        Self { permissions: Vec::new() }
    }
}

impl Storable for SystemPermissionIDList {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 1024 * 4, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

