
// src/core/state/drives/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{state::tags::types::TagStringValue, types::{ICPPrincipalString, PublicKeyICP}};


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