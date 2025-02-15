use std::fmt;

// src/core/state/drives/types.rs
use serde::{Serialize, Deserialize};

use crate::core::types::{ICPPrincipalString, PublicKeyICP};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DriveID(pub String);
impl fmt::Display for DriveID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct Drive {
    pub id: DriveID,
    pub name: String,
    pub icp_principal: ICPPrincipalString,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
}   
