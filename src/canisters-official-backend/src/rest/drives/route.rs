// src/rest/drives/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const DRIVES_GET_PATH: &str =                   genroute!("/drives/get/{drive_id}");
pub const DRIVES_LIST_PATH: &str =                  genroute!("/drives/list");
pub const DRIVES_UPSERT_PATH: &str =                genroute!("/drives/upsert");
pub const DRIVES_DELETE_PATH: &str =                genroute!("/drives/delete");
pub const ORG_SNAPSHOT_PATH: &str =                 genroute!("/organization/snapshot");
pub const ORG_REPLAY_PATH: &str =                   genroute!("/organization/replay");
pub const ORG_SEARCH_PATH: &str =                   genroute!("/organization/search");
pub const ORG_REINDEX_PATH: &str =                  genroute!("/organization/reindex");
pub const ORG_EXTERNAL_ID_PATH: &str =              genroute!("/organization/external_id");
pub const ORG_TRANSFER_OWNERSHIP_PATH: &str =       genroute!("/organization/transfer_ownership");
pub const ORG_WHOAMI_PATH: &str =                   genroute!("/organization/whoami");

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
            ORG_SNAPSHOT_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::snapshot_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_REPLAY_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::replay_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_SEARCH_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::search_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_REINDEX_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::reindex_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_EXTERNAL_ID_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::external_id_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_TRANSFER_OWNERSHIP_PATH,
            // transfering ownership requires owner call this route twice with the same body at least 24 hours apart
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::transfer_ownership_drive_handler(req, params)),
        ),
        (
            "GET",
            ORG_WHOAMI_PATH,
            |req, params| Box::pin(crate::rest::drives::handler::drives_handlers::whoami_drive_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}

