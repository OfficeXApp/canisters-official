// src/rest/templates/route.rs

use crate::debug_log;

use crate::rest::router;
use crate::types::RouteHandler;

pub const TEMPLATES_GET_PATH: &str = "/templates/get/{id}";
pub const TEMPLATES_LIST_PATH: &str = "/templates/list";
pub const TEMPLATES_UPSERT_PATH: &str = "/templates/upsert";
pub const TEMPLATES_DELETE_PATH: &str = "/templates/delete";
pub const WILDCARD_PATH: &str = "/*";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_query_routes() {
    let query_routes: &[HandlerEntry] = &[
        (
            "POST",
            TEMPLATES_LIST_PATH,
            crate::rest::templates::handler::templates_handlers::query_handler,
        ),
        (
            "GET",
            TEMPLATES_GET_PATH,
            crate::rest::templates::handler::templates_handlers::query_handler,
        ),
        (
            "POST",
            TEMPLATES_UPSERT_PATH,
            crate::rest::templates::handler::templates_handlers::upgrade_to_update_call_handler,
        )
    ];

    for &(method, path, handler) in query_routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_query_route(method, path, handler);
    }

    let wildcard_methods = ["GET", "HEAD", "PUT", "POST", "OPTIONS", "TRACE", "CONNECT"];
    for &method in &wildcard_methods {
        router::insert_query_route(
            method,
            WILDCARD_PATH,
            crate::rest::templates::handler::templates_handlers::query_handler,
        );
    }
}

pub fn init_update_routes() {
    let update_routes: &[HandlerEntry] = &[
        (
            "POST",
            TEMPLATES_UPSERT_PATH,
            crate::rest::templates::handler::templates_handlers::create_template_handler,
        ),
        (
            "POST",
            TEMPLATES_DELETE_PATH,
            crate::rest::templates::handler::templates_handlers::delete_template_handler,
        )
    ];

    for &(method, path, handler) in update_routes {
        router::insert_update_route(method, path, handler);
    }

    let wildcard_methods = ["GET", "HEAD", "PUT", "POST", "OPTIONS", "TRACE", "CONNECT"];
    for &method in &wildcard_methods {
        router::insert_update_route(
            method,
            WILDCARD_PATH,
            crate::rest::templates::handler::templates_handlers::no_update_call_handler,
        );
    }
}