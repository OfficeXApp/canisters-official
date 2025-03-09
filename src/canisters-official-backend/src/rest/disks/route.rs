// src/rest/disks/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const DISKS_GET_PATH: &str =        genroute!("/disks/get/{disk_id}");
pub const DISKS_LIST_PATH: &str =       genroute!("/disks/list");
pub const DISKS_CREATE_PATH: &str =     genroute!("/disks/create");
pub const DISKS_UPDATE_PATH: &str =     genroute!("/disks/update");
pub const DISKS_DELETE_PATH: &str =     genroute!("/disks/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            DISKS_GET_PATH,
            |req, params| Box::pin(crate::rest::disks::handler::disks_handlers::get_disk_handler(req, params)),
        ),
        (
            "POST",
            DISKS_LIST_PATH,
            |req, params| Box::pin(crate::rest::disks::handler::disks_handlers::list_disks_handler(req, params)),
        ),
        (
            "POST",
            DISKS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::disks::handler::disks_handlers::create_disk_handler(req, params)),
        ),
        (
            "POST",
            DISKS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::disks::handler::disks_handlers::update_disk_handler(req, params)),
        ),
        (
            "POST",
            DISKS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::disks::handler::disks_handlers::delete_disk_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}