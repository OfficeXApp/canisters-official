// src/core/state/contacts/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::core::{state::{tags::types::TagStringValue, teams::types::TeamID}, types::{ICPPrincipalString, PublicKeyICP, UserID}};


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Contact {
    pub id: UserID,
    pub nickname: String,
    pub public_note: String,
    pub private_note: Option<String>,
    pub evm_public_address: String,
    pub icp_principal: ICPPrincipalString,
    pub teams: Vec<TeamID>,
    pub tags: Vec<TagStringValue>,
}   
