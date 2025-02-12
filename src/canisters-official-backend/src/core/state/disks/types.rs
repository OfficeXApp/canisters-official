// src/core/state/disks/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiskID(pub String);
impl fmt::Display for DiskID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    pub id: DiskID,
    pub name: String,
    pub disk_type: DiskTypeEnum,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
    pub auth_json: Option<String>,
    pub external_id: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsBucketAuth {
    pub(crate) endpoint: String,
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
    pub(crate) bucket: String,
    pub(crate) region: String,  
}

