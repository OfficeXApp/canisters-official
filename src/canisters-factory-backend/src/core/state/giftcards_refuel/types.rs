
// src/core/state/drives/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{ state::giftcards_spawnorg::types::{DriveID, DriveRESTUrlEndpoint}, types::{ICPPrincipalString, PublicKeyICP, UserID}};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct GiftcardRefuelID(pub String);
impl fmt::Display for GiftcardRefuelID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct FactoryRefuelHistoryRecord {
    pub note: String,
    pub giftcard_id: GiftcardRefuelID,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
    pub icp_principal: ICPPrincipalString,
}

// Define a struct to track deployment history
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct GiftcardRefuel {
    pub id: GiftcardRefuelID,
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub timestamp_ms: u64,
    pub external_id: String, // eg. stripe charge id or evm tx hash
    pub redeemed: bool,
}
