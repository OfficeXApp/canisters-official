// src/core/state/disks/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use std::fmt;

use crate::core::{api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}}, types::UserID};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DiskID(pub String);
impl fmt::Display for DiskID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Disk {
    pub id: DiskID,
    pub name: String,
    pub disk_type: DiskTypeEnum,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
    pub auth_json: Option<String>,
    pub tags: Vec<TagStringValue>,
    pub created_at: u64,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}


impl Disk {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Disks),
            PermissionGranteeID::User(user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        let permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(user_id.clone())
        );
        let has_edit_permissions = permissions.contains(&SystemPermissionType::Edit) || table_permissions.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.auth_json = None;
                redacted.private_note = None;
            }
        }
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


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SerdeDiff)]
pub enum DiskTypeEnum {
    BrowserCache,
    LocalSSD,
    AwsBucket,
    StorjWeb3,
    IcpCanister,
}
impl fmt::Display for DiskTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiskTypeEnum::BrowserCache => write!(f, "BrowserCache"),
            DiskTypeEnum::LocalSSD => write!(f, "LocalSSD"),
            DiskTypeEnum::AwsBucket => write!(f, "AwsBucket"),
            DiskTypeEnum::StorjWeb3 => write!(f, "StorjWeb3"),
            DiskTypeEnum::IcpCanister => write!(f, "IcpCanister"),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct AwsBucketAuth {
    pub(crate) endpoint: String,
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
    pub(crate) bucket: String,
    pub(crate) region: String,  
}

