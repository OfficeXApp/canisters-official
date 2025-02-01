// src/types.rs
use serde::{Deserialize, Serialize};
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;


pub type RouteHandler = for<'a> fn(&'a HttpRequest, &'a Params) -> HttpResponse<'static>;


#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CreateType {
    Create,
}


#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateType {
    Update,
}