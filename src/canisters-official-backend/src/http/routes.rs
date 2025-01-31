// http/routes.rs
use crate::{
    handlers::todo_handlers::*,
    types::RouteHandler,
};
use ic_http_certification::{HttpRequest, HttpResponse, Method};
use matchit::Router;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static QUERY_ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
    static UPDATE_ROUTER: RefCell<HashMap<String, Router<RouteHandler>>> = RefCell::new(HashMap::new());
}

// Route paths
pub const TODOS_PATH: &str = "/todos";
pub const TODOS_ID_PATH: &str = "/todos/{id}";
pub const WILDCARD_PATH: &str = "/{*p}";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    init_query_routes();
    init_update_routes();
}

fn init_query_routes() {
    // Define query routes with their handlers
    let query_routes: &[HandlerEntry] = &[
        ("POST", TODOS_PATH, upgrade_to_update_call_handler as RouteHandler),
        ("PATCH", TODOS_ID_PATH, upgrade_to_update_call_handler as RouteHandler),
        ("DELETE", TODOS_ID_PATH, upgrade_to_update_call_handler as RouteHandler),
        ("GET", WILDCARD_PATH, query_handler as RouteHandler),
    ];

    // Register all query routes
    for &(method, path, handler) in query_routes {
        insert_query_route(method, path, handler);
    }

    // Add standard methods that return query handler
    for method in ["HEAD", "PUT", "OPTIONS", "TRACE", "CONNECT"] {
        insert_query_route(method, WILDCARD_PATH, query_handler as RouteHandler);
    }
}

fn init_update_routes() {
    // Define update routes with their handlers
    let update_routes: &[HandlerEntry] = &[
        ("POST", TODOS_PATH, create_todo_item_handler as RouteHandler),
        ("PATCH", TODOS_ID_PATH, update_todo_item_handler as RouteHandler),
        ("DELETE", TODOS_ID_PATH, delete_todo_item_handler as RouteHandler),
        ("GET", WILDCARD_PATH, no_update_call_handler as RouteHandler),
    ];

    // Register all update routes
    for &(method, path, handler) in update_routes {
        insert_update_route(method, path, handler);
    }

    // Add standard methods that return no update handler
    for method in ["HEAD", "PUT", "OPTIONS", "TRACE", "CONNECT"] {
        insert_update_route(method, WILDCARD_PATH, no_update_call_handler as RouteHandler);
    }
}

pub fn handle_query_request(req: HttpRequest) -> HttpResponse<'static> {
    let req_path = req.get_path().expect("Failed to get req path");
    
    QUERY_ROUTER.with_borrow(|query_router| {
        let method_router = query_router
            .get(&req.method().as_str().to_uppercase())
            .unwrap();
        let handler_match = method_router.at(&req_path).unwrap();
        let handler = handler_match.value;

        handler(&req, &handler_match.params)
    })
}

pub fn handle_update_request(req: HttpRequest) -> HttpResponse<'static> {
    let req_path = req.get_path().expect("Failed to get req path");
    
    UPDATE_ROUTER.with_borrow(|update_router| {
        let method_router = update_router
            .get(&req.method().as_str().to_uppercase())
            .unwrap();
        let handler_match = method_router.at(&req_path).unwrap();
        let handler = handler_match.value;

        handler(&req, &handler_match.params)
    })
}

fn insert_query_route(method: &str, path: &str, route_handler: RouteHandler) {
    QUERY_ROUTER.with_borrow_mut(|query_router| {
        let router = query_router.entry(method.to_string()).or_default();
        router.insert(path, route_handler).unwrap();
    });
}

fn insert_update_route(method: &str, path: &str, route_handler: RouteHandler) {
    UPDATE_ROUTER.with_borrow_mut(|update_router| {
        let router = update_router.entry(method.to_string()).or_default();
        router.insert(path, route_handler).unwrap();
    });
}