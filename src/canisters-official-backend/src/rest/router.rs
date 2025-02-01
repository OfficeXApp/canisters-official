// src/rest/router.rs
use crate::debug_log;
use crate::types::RouteHandler;
use ic_http_certification::{HttpRequest, HttpResponse};
use matchit::Router;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
}

pub fn init_routes() {
    crate::rest::templates::route::init_routes();
}

pub fn handle_request(req: HttpRequest) -> HttpResponse<'static> {
    let req_path = req.get_path().expect("Failed to get req path");
    debug_log!("Handling request for path: {}", req_path);
    
    ROUTER.with_borrow(|router| {
        let method_router = router
            .get(&req.method().as_str().to_uppercase())
            .unwrap();
        
        let handler_match = method_router.at(&req_path).unwrap();
        let handler = handler_match.value;

        handler(&req, &handler_match.params)
    })
}

pub(crate) fn insert_route(method: &str, path: &str, route_handler: RouteHandler) {
    ROUTER.with_borrow_mut(|router| {
        let method_router = router.entry(method.to_string()).or_default();
        method_router.insert(path, route_handler).unwrap();
    });
}