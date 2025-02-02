// src/rest/router.rs
use crate::{debug_log, rest::helpers};
use crate::types::RouteHandler;
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Router;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
}

const WILDCARD_PATH: &str = "/*";


pub fn not_found_handler(req: &HttpRequest, _params: &matchit::Params) -> HttpResponse<'static> {
    debug_log!("Path not found: {}", req.url());
    helpers::not_found_response()
}

pub fn init_routes() {
    crate::rest::templates::route::init_routes();
    crate::rest::apikeys::route::init_routes();
    crate::rest::webhooks::route::init_routes();

    // Handle not found for all methods with wildcard routes
    let wildcard_methods = ["GET", "HEAD", "PUT", "POST", "DELETE", "OPTIONS", "TRACE", "CONNECT"];
    for &method in &wildcard_methods {
        insert_route(
            method,
            WILDCARD_PATH,
            not_found_handler,
        );
    }
}

pub fn handle_request(req: HttpRequest) -> HttpResponse<'static> {
    let req_path = match req.get_path() {
        Ok(path) => path,
        Err(_) => return helpers::not_found_response(),
    };
    
    debug_log!("Handling request for path: {}", req_path);
    
    ROUTER.with_borrow(|router| {
        // Get the router for this HTTP method
        let method_router = match router.get(&req.method().as_str().to_uppercase()) {
            Some(router) => router,
            None => return helpers::not_found_response(),
        };
        
        // Try to match the route
        match method_router.at(&req_path) {
            Ok(handler_match) => {
                let handler = handler_match.value;
                handler(&req, &handler_match.params)
            },
            Err(_) => helpers::not_found_response(),
        }
    })
}

pub(crate) fn insert_route(method: &str, path: &str, route_handler: RouteHandler) {
    ROUTER.with_borrow_mut(|router| {
        let method_router = router.entry(method.to_string()).or_default();
        method_router.insert(path, route_handler).unwrap();
    });
}