// src/rest/giftcards/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


// ROUTE_PREFIX
pub const GIFTCARDS_SPAWNORG_GET_PATH: &str =      genroute!("/giftcards/spawnorg/get/{giftcard_id}");
pub const GIFTCARDS_SPAWNORG_LIST_PATH: &str =     genroute!("/giftcards/spawnorg/list");
pub const GIFTCARDS_SPAWNORG_CREATE_PATH: &str =   genroute!("/giftcards/spawnorg/create");
pub const GIFTCARDS_SPAWNORG_UPDATE_PATH: &str =   genroute!("/giftcards/spawnorg/update");
pub const GIFTCARDS_SPAWNORG_DELETE_PATH: &str =   genroute!("/giftcards/spawnorg/delete");
pub const GIFTCARDS_SPAWNORG_REDEEM_PATH: &str =   genroute!("/giftcards/spawnorg/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            GIFTCARDS_SPAWNORG_GET_PATH,
            |req, params| Box::pin(crate::rest::giftcards_spawnorg::handler::giftcards_handlers::get_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_SPAWNORG_LIST_PATH,
            |req, params| Box::pin(crate::rest::giftcards_spawnorg::handler::giftcards_handlers::list_giftcards_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_SPAWNORG_CREATE_PATH,
            |req, params| Box::pin(crate::rest::giftcards_spawnorg::handler::giftcards_handlers::create_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_SPAWNORG_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::giftcards_spawnorg::handler::giftcards_handlers::update_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_SPAWNORG_DELETE_PATH,
            |req, params| Box::pin(crate::rest::giftcards_spawnorg::handler::giftcards_handlers::delete_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_SPAWNORG_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::giftcards_spawnorg::handler::giftcards_handlers::redeem_giftcard_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}