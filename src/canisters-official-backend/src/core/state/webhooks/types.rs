// src/core/state/webhooks/types.rs
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebhookID(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct WebhookItem {
    pub id: WebhookID,
    pub title: String,
    pub completed: bool,
}