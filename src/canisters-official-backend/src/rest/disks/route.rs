// src/rest/disks/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const DISKS_GET_PATH: &str = "/disks/get/{disk_id}";
pub const DISKS_LIST_PATH: &str = "/disks/list";
pub const DISKS_UPSERT_PATH: &str = "/disks/upsert";
pub const DISKS_DELETE_PATH: &str = "/disks/delete";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            DISKS_GET_PATH,
            crate::rest::disks::handler::disks_handlers::get_disk_handler,
        ),
        (
            "POST",
            DISKS_LIST_PATH,
            crate::rest::disks::handler::disks_handlers::list_disks_handler,
        ),
        (
            "POST",
            DISKS_UPSERT_PATH,
            crate::rest::disks::handler::disks_handlers::upsert_disk_handler,
        ),
        (
            "POST",
            DISKS_DELETE_PATH,
            crate::rest::disks::handler::disks_handlers::delete_disk_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}