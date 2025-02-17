// src/rest/router.rs
use crate::{debug_log, rest::helpers};
use crate::types::RouteHandler;
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use matchit::{Params, Router};
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
}

const WILDCARD_PATH: &str = "/*";


pub async fn not_found_handler<'a, 'k, 'v>(
    req: &'a HttpRequest<'a>, 
    _params: &'a Params<'k, 'v>
) -> HttpResponse<'static> {
    debug_log!("Path not found: {}", req.url());
    helpers::not_found_response()
}

pub fn init_routes() {
    crate::rest::templates::route::init_routes();
    crate::rest::api_keys::route::init_routes();
    crate::rest::webhooks::route::init_routes();
    crate::rest::contacts::route::init_routes();
    crate::rest::teams::route::init_routes();
    crate::rest::team_invites::route::init_routes();
    crate::rest::drives::route::init_routes();
    crate::rest::disks::route::init_routes();
    crate::rest::directory::route::init_routes();
    crate::rest::permissions::route::init_routes();

    debug_log!("Initializing routes...");

    insert_route(
        "OPTIONS",
        "/*",
        |req, params| Box::pin(handle_options_request(req, params)),
    );

    // Handle not found for all methods with wildcard routes
    let wildcard_methods = ["GET", "HEAD", "PUT", "POST", "DELETE", "TRACE", "CONNECT"];
    for &method in &wildcard_methods {
        insert_route(
            method,
            WILDCARD_PATH,
            |req, params| Box::pin(not_found_handler(req, params)),
        );
    }
}

pub async fn handle_request(req: HttpRequest<'_>) -> HttpResponse<'static> {

    debug_log!("Handling request...");

    let req_path: String = match req.get_path() {
        Ok(path) => path,
        Err(_) => return helpers::not_found_response(),
    };
    
    debug_log!("Handling request for path: {}", req_path);
    
    let method_router = ROUTER.with_borrow(|router| {
        match router.get(&req.method().as_str().to_uppercase()) {
            Some(router) => Ok(router.clone()), // Assuming router implements Clone
            None => Err(()),
        }
    });
    
    let method_router = match method_router {
        Ok(router) => router,
        Err(_) => return helpers::not_found_response(),
    };
    
    // Now use method_router outside with_borrow
    match method_router.at(&req_path) {
        Ok(handler_match) => {
            let handler = handler_match.value;
            let future = handler(&req, &handler_match.params);
            future.await
        },
        Err(_) => helpers::not_found_response()
    }
}

pub(crate) fn insert_route(method: &str, path: &str, route_handler: RouteHandler) {
    ROUTER.with_borrow_mut(|router| {
        let method_router = router.entry(method.to_string()).or_default();
        method_router.insert(path, route_handler).unwrap();
    });
}


pub async fn handle_options_request<'a, 'k, 'v>(
    _req: &'a HttpRequest<'a>, 
    _params: &'a Params<'k, 'v>
) -> HttpResponse<'static> {
    let headers = vec![
        ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
        ("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()),
        ("Access-Control-Allow-Headers".to_string(), "Content-Type, Api-Key".to_string()),
        ("Access-Control-Max-Age".to_string(), "86400".to_string()),
    ];

    HttpResponse::builder()
        .with_status_code(StatusCode::NO_CONTENT)
        .with_headers(headers)
        .build()
}
