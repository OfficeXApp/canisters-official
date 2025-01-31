// src/types.rs

use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TemplateItem {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

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

pub type CreateTemplateResponse<'a> = TemplateResponse<'a, TemplateItem>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateTemplateResponse<'a> = TemplateResponse<'a, ()>;

pub type DeleteTemplateResponse<'a> = TemplateResponse<'a, ()>;

pub type ListTemplatesResponse<'a> = TemplateResponse<'a, Vec<TemplateItem>>;

pub type ErrorResponse<'a> = TemplateResponse<'a, ()>;

pub type RouteHandler = for<'a> fn(&'a HttpRequest, &'a Params) -> HttpResponse<'static>;