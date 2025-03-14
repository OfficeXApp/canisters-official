// src/rest/labels/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const LABELS_GET_PATH: &str =         genroute!("/labels/get/{id}");
pub const LABELS_LIST_PATH: &str =        genroute!("/labels/list");
pub const LABELS_CREATE_PATH: &str =      genroute!("/labels/create");
pub const LABELS_UPDATE_PATH: &str =      genroute!("/labels/update");
pub const LABELS_DELETE_PATH: &str =      genroute!("/labels/delete");
pub const LABELS_RESOURCE_PATH: &str =    genroute!("/labels/pin");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            LABELS_GET_PATH,
            |req, params| Box::pin(crate::rest::labels::handler::labels_handlers::get_label_handler(req, params)),
        ),
        (
            "POST",
            LABELS_LIST_PATH,
            |req, params| Box::pin(crate::rest::labels::handler::labels_handlers::list_labels_handler(req, params)),
        ),
        (
            "POST",
            LABELS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::labels::handler::labels_handlers::create_label_handler(req, params)),
        ),
        (
            "POST",
            LABELS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::labels::handler::labels_handlers::update_label_handler(req, params)),
        ),
        (
            "POST",
            LABELS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::labels::handler::labels_handlers::delete_label_handler(req, params)),
        ),
        (
            "POST",
            LABELS_RESOURCE_PATH,
            |req, params| Box::pin(crate::rest::labels::handler::labels_handlers::label_pin_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}