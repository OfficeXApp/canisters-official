// src/core/state/disks/types.rs
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiskID(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Disk {
    pub id: DiskID,
    pub name: String,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
    pub auth_json: Option<String>,
    pub disk_type: Option<DiskTypeEnum>,
    pub slug: Option<String>,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum DiskTypeEnum {
    BrowserCache,
    LocalSSD,
    AwsBucket,
    StorjWeb3,
    IcpCanister,
}