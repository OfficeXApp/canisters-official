// src/rest/helpers.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use serde_json::json;
use std::borrow::Cow;
use url::form_urlencoded;

pub fn create_response(status_code: StatusCode, body: String) -> HttpResponse<'static> {
    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
        ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
        ("Access-Control-Allow-Headers".to_string(), "Content-Type, Api-Key, Authorization".to_string()),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
    ];
    
    HttpResponse::builder()
        .with_status_code(status_code)
        .with_headers(headers)
        .with_body(Cow::Owned(body.into_bytes()))
        .build()
}


pub fn not_found_response() -> HttpResponse<'static> {
    let error_payload = json!({
        "err": {
            "code": 404,
            "message": "REST API route not found"
        }
    });

    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
        ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
        ("Access-Control-Allow-Headers".to_string(), "Content-Type, Api-Key, Authorization".to_string()),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
    ];

    HttpResponse::builder()
        .with_status_code(StatusCode::NOT_FOUND)
        .with_headers(headers)
        .with_body(Cow::Owned(error_payload.to_string().into_bytes()))
        .build()
}


/// Use `url::form_urlencoded` to parse query string into key-value pairs.
pub fn parse_query_string(query: &str) -> std::collections::HashMap<String, String> {
    form_urlencoded::parse(query.as_bytes()).into_owned().collect()
}