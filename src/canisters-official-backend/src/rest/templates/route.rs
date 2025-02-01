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

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            TEMPLATES_GET_PATH,
            crate::rest::templates::handler::templates_handlers::get_template_handler,
        ),
        (
            "POST",
            TEMPLATES_LIST_PATH,
            crate::rest::templates::handler::templates_handlers::list_templates_handler,
        ),
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

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

    // Handle not found for all methods
    let wildcard_methods = ["GET", "HEAD", "PUT", "POST", "DELETE", "OPTIONS", "TRACE", "CONNECT"];
    for &method in &wildcard_methods {
        router::insert_route(
            method,
            WILDCARD_PATH,
            crate::rest::templates::handler::templates_handlers::not_found_handler,
        );
    }
}