// src/rest/drives/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::types::RouteHandler;

pub const DRIVES_GET_PATH: &str =                   genroute!("/drives/get/{drive_id}");
pub const DRIVES_LIST_PATH: &str =                  genroute!("/drives/list");
pub const DRIVES_UPSERT_PATH: &str =                genroute!("/drives/upsert");
pub const DRIVES_DELETE_PATH: &str =                genroute!("/drives/delete");
pub const DRIVES_SNAPSHOT_PATH: &str =              genroute!("/drives/snapshot");
pub const DRIVES_REPLAY_PATH: &str =                genroute!("/drives/replay");
pub const DRIVES_SEARCH_PATH: &str =                genroute!("/drives/search");
pub const DRIVES_REINDEX_PATH: &str =               genroute!("/drives/reindex");
pub const DRIVES_EXTERNAL_ID_PATH: &str =           genroute!("/drives/external_id");
pub const DRIVES_TRANSFER_OWNERSHIP_PATH: &str =    genroute!("/drives/transfer_ownership");

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
            DRIVES_UPSERT_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::upsert_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_DELETE_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::delete_drive_handler(req, params)),
        ),
        (
            "GET",
            DRIVES_SNAPSHOT_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::snapshot_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_REPLAY_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::replay_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_SEARCH_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::search_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_REINDEX_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::reindex_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_EXTERNAL_ID_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::external_id_drive_handler(req, params)),
        ),
        (
            "POST",
            DRIVES_TRANSFER_OWNERSHIP_PATH,
            // transfering ownership requires owner call this route twice with the same body at least 24 hours apart
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::transfer_ownership_drive_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}

