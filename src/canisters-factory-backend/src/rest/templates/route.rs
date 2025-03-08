// src/rest/templates/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const TEMPLATES_GET_PATH: &str =    genroute!("/templates/get/{id}");
pub const TEMPLATES_LIST_PATH: &str =   genroute!("/templates/list");
pub const TEMPLATES_UPSERT_PATH: &str = genroute!("/templates/upsert");
pub const TEMPLATES_DELETE_PATH: &str = genroute!("/templates/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            TEMPLATES_GET_PATH,
            |req, params| Box::pin(crate::rest::templates::handler::templates_handlers::get_template_handler(req, params)),
        ),
        (
            "POST",
            TEMPLATES_LIST_PATH,
            |req, params| Box::pin(crate::rest::templates::handler::templates_handlers::list_templates_handler(req, params)),
        ),
        (
            "POST",
            TEMPLATES_UPSERT_PATH,
            |req, params| Box::pin(crate::rest::templates::handler::templates_handlers::upsert_template_handler(req, params)),
        ),
        (
            "POST",
            TEMPLATES_DELETE_PATH,
            |req, params| Box::pin(crate::rest::templates::handler::templates_handlers::delete_template_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}