// src/rest/webhooks/types.rs

use serde::{Deserialize, Serialize};

use crate::core::state::webhooks::types::{WebhookID, WebhookItem};

#[derive(Debug, Clone, Serialize)]
pub enum WebhookResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> WebhookResponse<'a, T> {
    pub fn ok(data: &'a T) -> WebhookResponse<T> {
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn unauthorized() -> Self {
        Self::err(401, "Unauthorized".to_string())
    }

    pub fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize value")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWebhookRequest {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWebhookRequest {
    pub id: WebhookID,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedWebhookData {
    pub id: WebhookID,
    pub deleted: bool
}

pub type DeleteWebhookResponse<'a> = WebhookResponse<'a, DeletedWebhookData>;

pub type CreateWebhookResponse<'a> = WebhookResponse<'a, WebhookItem>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebhookRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateWebhookResponse<'a> = WebhookResponse<'a, ()>;

pub type ListWebhooksResponse<'a> = WebhookResponse<'a, Vec<WebhookItem>>;

pub type GetWebhookResponse<'a> = WebhookResponse<'a, WebhookItem>;

pub type ErrorResponse<'a> = WebhookResponse<'a, ()>;