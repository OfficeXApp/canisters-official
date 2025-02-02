// src/core/state/templates/types.rs
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateID(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct TemplateItem {
    pub id: TemplateID,
    pub title: String,
    pub completed: bool,
}