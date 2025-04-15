
// src/core/state/drives/types.rs
use std::{borrow::Cow, fmt};
use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::{core::{api::permissions::system::check_system_permissions, state::{permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, labels::types::{redact_label, LabelStringValue}}, types::{ICPPrincipalString, PublicKeyICP, UserID}}, rest::drives::types::DriveFE};

use super::state::state::OWNER_ID;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct DriveID(pub String);
impl fmt::Display for DriveID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for DriveID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DriveID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DriveID")
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct InboxNotifID(pub String);
impl fmt::Display for InboxNotifID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct Drive {
    pub id: DriveID,
    pub name: String,
    pub icp_principal: ICPPrincipalString,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: DriveRESTUrlEndpoint,
    pub last_indexed_ms: Option<u64>,
    pub created_at: u64,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}   

impl Storable for Drive {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize Drive");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize Drive")
    }
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


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct SpawnRedeemCode(pub String);

impl Storable for SpawnRedeemCode {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize SpawnRedeemCode");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize SpawnRedeemCode")
    }
}
impl fmt::Display for SpawnRedeemCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct FactorySpawnHistoryRecord {
    pub owner_id: UserID,
    pub drive_id: DriveID,
    pub endpoint: String,
}

impl Storable for FactorySpawnHistoryRecord {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FactorySpawnHistoryRecord");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FactorySpawnHistoryRecord")
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct DriveStateDiffID(pub String);
impl fmt::Display for DriveStateDiffID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct DriveStateDiffString(pub String);  // base64 encoded diff
impl fmt::Display for DriveStateDiffString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType   )]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveStateDiffImplementationType {
    RustIcpCanister,
    JavascriptRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct StateChecksum(pub String);
impl fmt::Display for StateChecksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for StateChecksum {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize StateChecksum");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize StateChecksum")
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct DriveRESTUrlEndpoint(pub String);
impl fmt::Display for DriveRESTUrlEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for DriveRESTUrlEndpoint {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DriveRESTUrlEndpoint");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DriveRESTUrlEndpoint")
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
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



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord, CandidType)]
pub struct ExternalID(pub String);
impl fmt::Display for ExternalID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for ExternalID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize ExternalID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize ExternalID")
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord, CandidType)]
pub struct ExternalPayload(pub String);
impl fmt::Display for ExternalPayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for ExternalPayload {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize ExternalPayload");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize ExternalPayload")
    }
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize, SerdeDiff)]
pub struct StringVec {
    pub items: Vec<String>,
}

impl StringVec {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn with_item(item: String) -> Self {
        Self { items: vec![item] }
    }
    
    pub fn push(&mut self, item: String) {
        self.items.push(item);
    }
    
    pub fn retain<F>(&mut self, f: F) 
    where 
        F: FnMut(&String) -> bool 
    {
        self.items.retain(f);
    }
    
    pub fn contains(&self, item: &str) -> bool {
        self.items.iter().any(|i| i == item)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.items.iter()
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// Implement conversion between Vec<String> and StringVec
impl From<Vec<String>> for StringVec {
    fn from(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl From<StringVec> for Vec<String> {
    fn from(list: StringVec) -> Self {
        list.items
    }
}

// Implement Storable for StringVec
impl Storable for StringVec {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 1024, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize StringVec");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize StringVec")
    }
}