// src/rest/giftcards/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


// ROUTE_PREFIX
pub const GIFTCARDS_GET_PATH: &str =      genroute!("/giftcards/get/{giftcard_id}");
pub const GIFTCARDS_LIST_PATH: &str =     genroute!("/giftcards/list");
pub const GIFTCARDS_UPSERT_PATH: &str =   genroute!("/giftcards/upsert");
pub const GIFTCARDS_DELETE_PATH: &str =   genroute!("/giftcards/delete");
pub const GIFTCARDS_REDEEM_PATH: &str =   genroute!("/giftcards/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            GIFTCARDS_GET_PATH,
            |req, params| Box::pin(crate::rest::giftcards::handler::giftcards_handlers::get_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_LIST_PATH,
            |req, params| Box::pin(crate::rest::giftcards::handler::giftcards_handlers::list_giftcards_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_UPSERT_PATH,
            |req, params| Box::pin(crate::rest::giftcards::handler::giftcards_handlers::upsert_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::giftcards::handler::giftcards_handlers::delete_giftcard_handler(req, params)),
        ),
        (
            "POST",
            GIFTCARDS_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::giftcards::handler::giftcards_handlers::redeem_giftcard_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}