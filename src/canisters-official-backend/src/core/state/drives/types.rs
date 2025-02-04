// src/core/state/drives/types.rs
use serde::{Serialize, Deserialize};

use crate::core::types::{PublicKeyBLS, UserID};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DriveID(pub PublicKeyBLS);



#[derive(Debug, Clone, Serialize)]
pub struct Drive {
    pub id: DriveID,
    pub name: String,
    pub owner_id: Option<UserID>,
    pub gas_remaining: Option<u64>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
}   
