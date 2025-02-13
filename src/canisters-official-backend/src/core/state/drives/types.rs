// src/core/state/drives/types.rs
use serde::{Serialize, Deserialize};

use crate::core::types::{ICPPrincipalString, PublicKeyICP};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DriveID(pub String);



#[derive(Debug, Clone, Serialize)]
pub struct Drive {
    pub id: DriveID,
    pub name: String,
    pub icp_principal: ICPPrincipalString,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
}   
