
// src/core/state/drives/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::{core::{api::permissions::system::check_system_permissions, state::{permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}}, types::{ICPPrincipalString, PublicKeyICP, UserID}}, rest::drives::types::DriveFE};

use super::state::state::OWNER_ID;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveID(pub String);
impl fmt::Display for DriveID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Drive {
    pub id: DriveID,
    pub name: String,
    pub icp_principal: ICPPrincipalString,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: DriveRESTUrlEndpoint,
    pub last_indexed_ms: Option<u64>,
    pub created_at: u64,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}   

impl Drive {

    pub fn cast_fe(&self, user_id: &UserID) -> DriveFE {
        let drive = self.clone();
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Drive(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Drives),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        DriveFE {
            drive,
            permission_previews
        }.redacted(user_id)
    }

    
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct SpawnRedeemCode(pub String);

// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct FactorySpawnHistoryRecord {
    pub owner_id: UserID,
    pub drive_id: DriveID,
    pub endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveStateDiffID(pub String);
impl fmt::Display for DriveStateDiffID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveStateDiffString(pub String);  // base64 encoded diff
impl fmt::Display for DriveStateDiffString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveStateDiffImplementationType {
    RustIcpCanister,
    JavascriptRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct StateChecksum(pub String);
impl fmt::Display for StateChecksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveRESTUrlEndpoint(pub String);
impl fmt::Display for DriveRESTUrlEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiffRecord {
    pub id: DriveStateDiffID,
    pub timestamp_ns: u64,
    pub notes: Option<String>,
    pub drive_id: DriveID,
    pub endpoint_url: DriveRESTUrlEndpoint,
    pub implementation: DriveStateDiffImplementationType,
    pub diff_forward: DriveStateDiffString,
    pub diff_backward: DriveStateDiffString,
    pub checksum_forward: StateChecksum,
    pub checksum_backward: StateChecksum,
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ExternalID(pub String);
impl fmt::Display for ExternalID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct ExternalPayload(pub String);
impl fmt::Display for ExternalPayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
