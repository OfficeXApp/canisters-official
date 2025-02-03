// src/types.rs
use serde::{Deserialize, Serialize};
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;


pub type RouteHandler = for<'a> fn(&'a HttpRequest, &'a Params) -> HttpResponse<'static>;

