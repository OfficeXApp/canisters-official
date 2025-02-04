// src/rest/webhooks/types.rs

use serde::{Deserialize, Serialize};
use crate::core::state::webhooks::types::WebhookEventLabel;
use crate::core::state::webhooks::types::{WebhookID, Webhook};

#[derive(Debug, Clone, Serialize)]
pub enum WebhookResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> WebhookResponse<'a, T> {
    pub fn ok(data: &'a T) -> WebhookResponse<'a, T> { 
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
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}



#[derive(Debug, Clone, Deserialize)]
pub struct ListWebhooksRequestBody {
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize)]
pub struct ListWebhooksResponseData {
    pub items: Vec<Webhook>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateWebhookRequestBody {
    pub alt_index: String,
    pub url: String,
    pub event: String,
    pub signature: Option<String>,
    pub description: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebhookRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertWebhookRequestBody {
    Create(CreateWebhookRequestBody),
    Update(UpdateWebhookRequestBody),
}


#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWebhookRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedWebhookData {
    pub id: WebhookID,
    pub deleted: bool
}


pub type GetWebhookResponse<'a> = WebhookResponse<'a, Webhook>;
pub type ListWebhooksResponse<'a> = WebhookResponse<'a, ListWebhooksResponseData>;
pub type CreateWebhookResponse<'a> = WebhookResponse<'a, Webhook>;
pub type UpdateWebhookResponse<'a> = WebhookResponse<'a, Webhook>;
pub type DeleteWebhookResponse<'a> = WebhookResponse<'a, DeletedWebhookData>;
pub type ErrorResponse<'a> = WebhookResponse<'a, ()>;