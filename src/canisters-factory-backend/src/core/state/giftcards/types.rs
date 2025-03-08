
// src/core/state/drives/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{ types::{ICPPrincipalString, PublicKeyICP, UserID}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveID(pub String);
impl fmt::Display for DriveID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct GiftcardID(pub String);
impl fmt::Display for GiftcardID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
    pub giftcard_id: GiftcardID,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
}

// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Giftcard {
    pub id: GiftcardID,
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
    pub external_id: String, // eg. stripe charge id or evm tx hash
    pub redeemed: bool,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct DriveRESTUrlEndpoint(pub String);
impl fmt::Display for DriveRESTUrlEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
