// src/rest/drives/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const DRIVES_GET_PATH: &str =                   genroute!("/drives/get/{drive_id}");
pub const DRIVES_LIST_PATH: &str =                  genroute!("/drives/list");
pub const DRIVES_CREATE_PATH: &str =                genroute!("/drives/create");
pub const DRIVES_UPDATE_PATH: &str =                genroute!("/drives/update");
pub const DRIVES_DELETE_PATH: &str =                genroute!("/drives/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            DRIVES_GET_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::get_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_LIST_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::list_drives_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_CREATE_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::create_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::update_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_DELETE_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::delete_drive_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}

