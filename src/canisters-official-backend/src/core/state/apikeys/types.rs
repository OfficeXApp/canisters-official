
// src/core/state/apikeys/types.rs

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyItem {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}
