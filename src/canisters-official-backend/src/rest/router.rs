// src/rest/router.rs

use crate::debug_log;

use crate::types::RouteHandler;
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Router;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static QUERY_ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
    static UPDATE_ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
}

pub fn init_routes() {
    crate::rest::templates::route::init_query_routes();
    crate::rest::templates::route::init_update_routes();
}

pub fn handle_query_request(req: HttpRequest) -> HttpResponse<'static> {
    let req_path = req.get_path().expect("Failed to get req path");

    debug_log!("Handling query request for path: {}", req_path);
    
    QUERY_ROUTER.with_borrow(|query_router| {
        debug_log!("Query router: {:?}", query_router);
        let method_router = query_router
            .get(&req.method().as_str().to_uppercase())
            .unwrap();
        debug_log!("Method router: {:?}", method_router);
        let handler_match = method_router.at(&req_path).unwrap();
        let handler = handler_match.value;

        handler(&req, &handler_match.params)
    })
}

pub fn handle_update_request(req: HttpRequest) -> HttpResponse<'static> {
    let req_path = req.get_path().expect("Failed to get req path");

    debug_log!("Handling update request for path: {}", req_path);
    
    UPDATE_ROUTER.with_borrow(|update_router| {
        debug_log!("Update router: {:?}", update_router);
        let method_router = update_router
            .get(&req.method().as_str().to_uppercase())
            .unwrap();
        debug_log!("Method router: {:?}", method_router);
        let handler_match = method_router.at(&req_path).unwrap();
        let handler = handler_match.value;

        handler(&req, &handler_match.params)
    })
}

pub(crate) fn insert_query_route(method: &str, path: &str, route_handler: RouteHandler) {
    QUERY_ROUTER.with_borrow_mut(|query_router| {
        let router = query_router.entry(method.to_string()).or_default();
        router.insert(path, route_handler).unwrap();
    });
}

pub(crate) fn insert_update_route(method: &str, path: &str, route_handler: RouteHandler) {
    UPDATE_ROUTER.with_borrow_mut(|update_router| {
        let router = update_router.entry(method.to_string()).or_default();
        router.insert(path, route_handler).unwrap();
    });
}