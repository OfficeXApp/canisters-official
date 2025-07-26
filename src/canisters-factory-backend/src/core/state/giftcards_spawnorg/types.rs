
// src/core/state/drives/types.rs
use std::{borrow::Cow, fmt};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{ types::{ICPPrincipalString, PublicKeyICP, UserID}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord)]
pub struct DriveID(pub String);
impl fmt::Display for DriveID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for DriveID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256 * 4,
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


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord)]
pub struct GiftcardSpawnOrgID(pub String);
impl fmt::Display for GiftcardSpawnOrgID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Storable for GiftcardSpawnOrgID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256,
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GiftcardSpawnOrgID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GiftcardSpawnOrgID")
    }
}


// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct FactorySpawnHistoryRecord {
    pub owner_id: UserID,
    pub drive_id: DriveID,
    pub endpoint: DriveRESTUrlEndpoint,
    pub version: String,
    pub note: String,
    pub giftcard_id: GiftcardSpawnOrgID,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
}

impl Storable for FactorySpawnHistoryRecord {
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024, // Adjust based on your needs
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

// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct GiftcardSpawnOrg {
    pub id: GiftcardSpawnOrgID,
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
    pub external_id: Option<String>, // eg. stripe charge id or evm tx hash
    pub redeemed: bool,
    pub disk_auth_json: Option<String>,
}


impl Storable for GiftcardSpawnOrg {
    const BOUND: Bound = Bound::Bounded {
        max_size: 2048, // Adjust based on your needs, increased for disk_auth_json
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GiftcardSpawnOrg");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GiftcardSpawnOrg")
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveRESTUrlEndpoint(pub String);
impl fmt::Display for DriveRESTUrlEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Storable for DriveRESTUrlEndpoint {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256,
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


// Similar to StringVec in your reference code, but for GiftcardSpawnOrgID
#[derive(Clone, Debug, Deserialize, Serialize, SerdeDiff)]
pub struct GiftcardSpawnOrgIDVec {
    pub items: Vec<GiftcardSpawnOrgID>,
}

impl GiftcardSpawnOrgIDVec {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn with_item(item: GiftcardSpawnOrgID) -> Self {
        Self { items: vec![item] }
    }
    
    pub fn push(&mut self, item: GiftcardSpawnOrgID) {
        self.items.push(item);
    }
    
    pub fn contains(&self, item: &GiftcardSpawnOrgID) -> bool {
        self.items.contains(item)
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Storable for GiftcardSpawnOrgIDVec {
    const BOUND: Bound = Bound::Bounded {
        max_size: 10240, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GiftcardSpawnOrgIDVec");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GiftcardSpawnOrgIDVec")
    }
}