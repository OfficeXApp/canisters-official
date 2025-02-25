use std::fmt;

// src/core/state/drives/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::types::{ICPPrincipalString, PublicKeyICP};


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
pub struct DriveStateDiffChecksum(pub String);
impl fmt::Display for DriveStateDiffChecksum {
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
pub struct DriveStateDiffRecord {
    pub id: DriveStateDiffID,
    pub timestamp_ns: u64,
    pub implementation: DriveStateDiffImplementationType,
    pub diff: DriveStateDiffString,
    pub notes: Option<String>,
    pub drive_id: DriveID,
    pub url_endpoint: DriveRESTUrlEndpoint,
    pub checksum: DriveStateDiffChecksum,
}
