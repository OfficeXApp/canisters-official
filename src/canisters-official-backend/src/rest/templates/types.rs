// src/rest/templates/types.rs

use serde::{Deserialize, Serialize};

use crate::core::{state::templates::types::{TemplateID, TemplateItem}, types::ClientSuggestedUUID};

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



pub type GetTemplateResponse<'a> = TemplateResponse<'a, TemplateItem>;

pub type ListTemplatesResponse<'a> = TemplateResponse<'a, Vec<TemplateItem>>;


#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateRequest {
    pub id: Option<ClientSuggestedUUID>,
    pub title: String,
}

pub type CreateTemplateResponse<'a> = TemplateResponse<'a, TemplateItem>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateTemplateResponse<'a> = TemplateResponse<'a, TemplateItem>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTemplateRequest {
    pub id: TemplateID,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedTemplateData {
    pub id: TemplateID,
    pub deleted: bool
}

pub type DeleteTemplateResponse<'a> = TemplateResponse<'a, DeletedTemplateData>;


pub type ErrorResponse<'a> = TemplateResponse<'a, ()>;