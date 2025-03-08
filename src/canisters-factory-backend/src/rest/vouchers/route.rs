// src/rest/vouchers/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


// ROUTE_PREFIX
pub const VOUCHERS_GET_PATH: &str =      genroute!("/vouchers/get/{voucher_id}");
pub const VOUCHERS_LIST_PATH: &str =     genroute!("/vouchers/list");
pub const VOUCHERS_UPSERT_PATH: &str =   genroute!("/vouchers/upsert");
pub const VOUCHERS_DELETE_PATH: &str =   genroute!("/vouchers/delete");
pub const VOUCHERS_REDEEM_PATH: &str =   genroute!("/vouchers/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            VOUCHERS_GET_PATH,
            |req, params| Box::pin(crate::rest::vouchers::handler::vouchers_handlers::get_voucher_handler(req, params)),
        ),
        (
            "POST",
            VOUCHERS_LIST_PATH,
            |req, params| Box::pin(crate::rest::vouchers::handler::vouchers_handlers::list_vouchers_handler(req, params)),
        ),
        (
            "POST",
            VOUCHERS_UPSERT_PATH,
            |req, params| Box::pin(crate::rest::vouchers::handler::vouchers_handlers::upsert_voucher_handler(req, params)),
        ),
        (
            "POST",
            VOUCHERS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::vouchers::handler::vouchers_handlers::delete_voucher_handler(req, params)),
        ),
        (
            "POST",
            VOUCHERS_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::vouchers::handler::vouchers_handlers::redeem_voucher_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}