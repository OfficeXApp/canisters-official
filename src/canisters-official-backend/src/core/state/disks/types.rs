// src/core/state/disks/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use std::fmt;

use crate::{core::{api::permissions::system::check_system_permissions, state::{directory::types::FolderID, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, labels::types::{redact_label, LabelStringValue}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::UserID}, rest::{disks::types::DiskFE, labels::types::LabelFE}};


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
    pub labels: Vec<LabelStringValue>,
    pub created_at: u64,
    pub root_folder: FolderID,
    pub trash_folder: FolderID,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}


impl Disk {
    pub fn cast_fe(&self, user_id: &UserID) -> DiskFE {
        let disk = self.clone();
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Disk(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Disks),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        DiskFE {
            disk,
            permission_previews
        }.redacted(user_id)
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiskTypeEnum {
    Browser_Cache,
    Local_SSD,
    Aws_Bucket,
    Storj_Web3,
    Icp_Canister,
}
impl fmt::Display for DiskTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiskTypeEnum::Browser_Cache => write!(f, "BROWSER_CACHE"),
            DiskTypeEnum::Local_SSD => write!(f, "LOCAL_SSD"),
            DiskTypeEnum::Aws_Bucket => write!(f, "AWS_BUCKET"),
            DiskTypeEnum::Storj_Web3 => write!(f, "STORJ_WEB3"),
            DiskTypeEnum::Icp_Canister => write!(f, "ICP_CANISTER"),
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

