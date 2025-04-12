// src/rest/giftcards/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


// ROUTE_PREFIX
pub const GIFTCARDS_REFUEL_GET_PATH: &str =      genroute!("/giftcards/refuel/get/{giftcard_id}");
pub const GIFTCARDS_REFUEL_LIST_PATH: &str =     genroute!("/giftcards/refuel/list");
pub const GIFTCARDS_REFUEL_UPSERT_PATH: &str =   genroute!("/giftcards/refuel/upsert");
pub const GIFTCARDS_REFUEL_DELETE_PATH: &str =   genroute!("/giftcards/refuel/delete");
pub const GIFTCARDS_REFUEL_REDEEM_PATH: &str =   genroute!("/giftcards/refuel/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            GIFTCARDS_REFUEL_GET_PATH,
            |req, params| Box::pin(crate::rest::giftcards_refuel::handler::giftcards_handlers::get_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_REFUEL_LIST_PATH,
            |req, params| Box::pin(crate::rest::giftcards_refuel::handler::giftcards_handlers::list_giftcards_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_REFUEL_UPSERT_PATH,
            |req, params| Box::pin(crate::rest::giftcards_refuel::handler::giftcards_handlers::upsert_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_REFUEL_DELETE_PATH,
            |req, params| Box::pin(crate::rest::giftcards_refuel::handler::giftcards_handlers::delete_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_REFUEL_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::giftcards_refuel::handler::giftcards_handlers::redeem_giftcard_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}