// src/core/state/contacts/types.rs
use serde::{Serialize, Deserialize};

use crate::core::{state::teams::types::TeamID, types::UserID};



#[derive(Debug, Clone, Serialize)]
pub struct Contact {
    pub id: UserID,
    pub nickname: String,
    pub public_note: String,
    pub private_note: Option<String>,
    pub evm_public_address: String,
    pub icp_principal: String,
    pub teams: Vec<TeamID>
}   
