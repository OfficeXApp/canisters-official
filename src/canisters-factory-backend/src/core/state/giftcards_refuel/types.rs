
// src/core/state/drives/types.rs
use std::{borrow::Cow, fmt};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{ state::giftcards_spawnorg::types::{DriveID, DriveRESTUrlEndpoint}, types::{ICPPrincipalString, PublicKeyICP, UserID}};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, Ord, PartialOrd)]
pub struct GiftcardRefuelID(pub String);
impl fmt::Display for GiftcardRefuelID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for GiftcardRefuelID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256,
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GiftcardRefuelID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GiftcardRefuelID")
    }
}


// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct FactoryRefuelHistoryRecord {
    pub note: String,
    pub giftcard_id: GiftcardRefuelID,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
    pub icp_principal: ICPPrincipalString,
}

impl Storable for FactoryRefuelHistoryRecord {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize FactoryRefuelHistoryRecord");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize FactoryRefuelHistoryRecord")
    }
}


// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct GiftcardRefuel {
    pub id: GiftcardRefuelID,
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
    pub external_id: String, // eg. stripe charge id or evm tx hash
    pub redeemed: bool,
}

impl Storable for GiftcardRefuel {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GiftcardRefuel");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GiftcardRefuel")
    }
}



// Similar to StringVec in your reference code, but for GiftcardRefuelID
#[derive(Clone, Debug, Deserialize, Serialize, SerdeDiff, Ord, PartialOrd, PartialEq, Eq)]
pub struct GiftcardRefuelIDVec {
    pub items: Vec<GiftcardRefuelID>,
}

impl GiftcardRefuelIDVec {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn with_item(item: GiftcardRefuelID) -> Self {
        Self { items: vec![item] }
    }
    
    pub fn push(&mut self, item: GiftcardRefuelID) {
        self.items.push(item);
    }
    
    pub fn contains(&self, item: &GiftcardRefuelID) -> bool {
        self.items.contains(item)
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Storable for GiftcardRefuelIDVec {
    const BOUND: Bound = Bound::Bounded {
        max_size: 10240, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GiftcardRefuelIDVec");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GiftcardRefuelIDVec")
    }
}