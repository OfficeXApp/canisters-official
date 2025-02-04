// src/rest/drives/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const DRIVES_GET_PATH: &str = "/drives/get/{drive_id}";
pub const DRIVES_LIST_PATH: &str = "/drives/list";
pub const DRIVES_UPSERT_PATH: &str = "/drives/upsert";
pub const DRIVES_DELETE_PATH: &str = "/drives/delete";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            DRIVES_GET_PATH,
            crate::rest::drives::handler::drives_handlers::get_drive_handler,
        ),
        (
            "POST",
            DRIVES_LIST_PATH,
            crate::rest::drives::handler::drives_handlers::list_drives_handler,
        ),
        (
            "POST",
            DRIVES_UPSERT_PATH,
            crate::rest::drives::handler::drives_handlers::upsert_drive_handler,
        ),
        (
            "POST",
            DRIVES_DELETE_PATH,
            crate::rest::drives::handler::drives_handlers::delete_drive_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}