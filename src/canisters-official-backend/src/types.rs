// src/types.rs
use serde::{Deserialize, Serialize};
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Params;


pub type RouteHandler = for<'a, 'k, 'v> fn(&'a HttpRequest<'a>, &'a Params<'k, 'v>) 
    -> core::pin::Pin<Box<dyn core::future::Future<Output = HttpResponse<'static>> + 'a>>;


    