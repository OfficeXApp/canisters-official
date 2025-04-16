// src/rest/helpers.rs
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use serde_json::json;
use std::borrow::Cow;
use url::{form_urlencoded, Url};

use crate::core::{state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE, types::UserID};

pub fn create_response(status_code: StatusCode, body: String) -> HttpResponse<'static> {
    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
        // ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
        // ("Access-Control-Allow-Headers".to_string(), "Content-Type, Api-Key".to_string()),
    ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
    ("Access-Control-Allow-Headers".to_string(), "Content-Type, Api-Key, Authorization".to_string()),
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

    // The headers for the "not found" response. We set the Access-Control-Allow-Origin
    // to "*" to allow requests from any origin. We also set the
    // Access-Control-Allow-Methods and Access-Control-Allow-Headers to allow
    // requests with the given methods and headers.
    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
        ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
        ("Access-Control-Allow-Headers".to_string(), "Content-Type, Api-Key, Authorization".to_string()),
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

pub fn update_last_online_at(userID: &UserID) {
    // Update the last online time for the user if there's a contact
    CONTACTS_BY_ID_HASHTABLE.with(|map| {
        // First get a mutable borrow
        let mut map_mut = map.borrow_mut();
        
        // Then check if the contact exists, get it, modify it, and insert it back
        if let Some(mut contact) = map_mut.get(userID).map(|data| data.clone()) {
            contact.last_online_ms = ic_cdk::api::time() / 1_000_000;
            map_mut.insert(userID.clone(), contact);
        }
    });
}