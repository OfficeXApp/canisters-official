// src/rest/organization/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const ORG_ABOUT_PATH: &str =                    genroute!("/organization/about");
pub const ORG_INBOX_PATH: &str =                    genroute!("/organization/inbox");
pub const ORG_SNAPSHOT_PATH: &str =                 genroute!("/organization/snapshot");
pub const ORG_REPLAY_PATH: &str =                   genroute!("/organization/replay");
pub const ORG_SEARCH_PATH: &str =                   genroute!("/organization/search");
pub const ORG_REINDEX_PATH: &str =                  genroute!("/organization/reindex");
pub const ORG_EXTERNAL_ID_PATH: &str =              genroute!("/organization/external_id");
pub const ORG_TRANSFER_OWNERSHIP_PATH: &str =       genroute!("/organization/transfer_ownership");
pub const ORG_WHOAMI_PATH: &str =                   genroute!("/organization/whoami");
pub const ORG_SUPERSWAP_PATH: &str =                genroute!("/organization/superswap_user");
pub const ORG_REDEEM_SPAWN_PATH: &str =             genroute!("/organization/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            ORG_ABOUT_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::about_drive_handler(req, params)),
        ),
        (
            "GET",
            ORG_SNAPSHOT_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::snapshot_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_INBOX_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::inbox_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_REPLAY_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::replay_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_SEARCH_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::search_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_REINDEX_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::reindex_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_EXTERNAL_ID_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::external_id_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_TRANSFER_OWNERSHIP_PATH,
            // transfering ownership requires owner call this route twice with the same body at least 24 hours apart
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::transfer_ownership_drive_handler(req, params)),
        ),
        (
            "GET",
            ORG_WHOAMI_PATH,
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::whoami_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_SUPERSWAP_PATH,
            // transfering ownership requires owner call this route twice with the same body at least 24 hours apart
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::superswap_userid_drive_handler(req, params)),
        ),
        (
            "POST",
            ORG_REDEEM_SPAWN_PATH,
            // transfering ownership requires owner call this route twice with the same body at least 24 hours apart
            |req, params| Box::pin(crate::rest::organization::handler::drives_handlers::redeem_organization_drive_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}

