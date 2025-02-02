// src/rest/helpers.rs
use ic_http_certification::{HttpResponse, StatusCode};
use serde_json::json;
use std::borrow::Cow;

pub fn create_response(status_code: StatusCode, body: String) -> HttpResponse<'static> {
    let headers = vec![(
        "Content-Type".to_string(),
        "application/json".to_string(),
    )];
    
    HttpResponse::builder()
        .with_status_code(status_code)
        .with_headers(headers)
        .with_body(Cow::Owned(body.into_bytes()))
        .build()
}

pub fn not_found_response(path: &str) -> HttpResponse<'static> {
    create_response(
        StatusCode::NOT_FOUND,
        json!({
            "error": format!("Path not found: {}", path),
        })
        .to_string(),
    )
}