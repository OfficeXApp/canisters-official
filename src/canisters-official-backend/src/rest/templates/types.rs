// src/rest/templates/types.rs

use serde::{Deserialize, Serialize};

use crate::core::state::templates::types::TemplateItem;

#[derive(Debug, Clone, Serialize)]
pub enum TemplateResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> TemplateResponse<'a, T> {
    pub fn ok(data: &'a T) -> TemplateResponse<T> {
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn not_allowed() -> Self {
        Self::err(405, "Method not allowed".to_string())
    }

    fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize value")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateRequest {
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTemplateRequest {
    pub id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedTemplateData {
    pub deleted_id: u32,
}

pub type DeleteTemplateResponse<'a> = TemplateResponse<'a, DeletedTemplateData>;

pub type CreateTemplateResponse<'a> = TemplateResponse<'a, TemplateItem>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateTemplateResponse<'a> = TemplateResponse<'a, ()>;

pub type ListTemplatesResponse<'a> = TemplateResponse<'a, Vec<TemplateItem>>;

pub type GetTemplateResponse<'a> = TemplateResponse<'a, TemplateItem>;

pub type ErrorResponse<'a> = TemplateResponse<'a, ()>;