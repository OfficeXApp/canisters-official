// src/core/state/templates/types.rs
use serde::{Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TemplateItem {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}