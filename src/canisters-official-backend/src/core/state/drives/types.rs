
// src/core/state/drives/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{api::permissions::system::check_system_permissions, state::{permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}}, types::{ICPPrincipalString, PublicKeyICP, UserID}};

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
    pub url_endpoint: DriveRESTUrlEndpoint,
    pub last_indexed_ms: Option<u64>,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}   

impl Drive {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Drives),
            PermissionGranteeID::User(user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        let permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(user_id.clone())
        );
        let has_edit_permissions = permissions.contains(&SystemPermissionType::Update) || table_permissions.contains(&SystemPermissionType::Update);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
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
    pub url_endpoint: DriveRESTUrlEndpoint,
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
