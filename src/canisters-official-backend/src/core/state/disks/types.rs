use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
// src/core/state/disks/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use std::{borrow::Cow, fmt};

use crate::{core::{api::permissions::system::check_system_permissions, state::{directory::types::FolderID, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, labels::types::{redact_label, LabelStringValue}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::UserID}, rest::{disks::types::DiskFE, labels::types::LabelFE}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct DiskID(pub String);
impl fmt::Display for DiskID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for DiskID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DiskID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DiskID")
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, PartialEq, Eq, PartialOrd, Ord)]
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
    pub endpoint: Option<String>,
}


impl Storable for Disk {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize Disk");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize Disk")
    }
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


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, SerdeDiff, CandidType)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiskTypeEnum {
    BrowserCache,
    LocalSsd,
    AwsBucket,
    StorjWeb3,
    IcpCanister,
}
impl fmt::Display for DiskTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DiskTypeEnum::BrowserCache => write!(f, "BROWSER_CACHE"),
            DiskTypeEnum::LocalSsd => write!(f, "LOCAL_SSD"),
            DiskTypeEnum::AwsBucket => write!(f, "AWS_BUCKET"),
            DiskTypeEnum::StorjWeb3 => write!(f, "STORJ_WEB3"),
            DiskTypeEnum::IcpCanister => write!(f, "ICP_CANISTER"),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct AwsBucketAuth {
    pub(crate) endpoint: String,
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
    pub(crate) bucket: String,
    pub(crate) region: String,  
}

