use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const PURCHASES_GET_PATH: &str = genroute!("/purchases/get/{purchase_id}");
pub const PURCHASES_LIST_PATH: &str = genroute!("/purchases/list");
pub const PURCHASES_CREATE_PATH: &str = genroute!("/purchases/create");
pub const PURCHASES_UPDATE_PATH: &str = genroute!("/purchases/update");
pub const PURCHASES_DELETE_PATH: &str = genroute!("/purchases/delete");


type HandlerEntry = (&'static str, &'static str, RouteHandler);

/// Initializes and registers all API routes related to Purchases.
pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            PURCHASES_GET_PATH,
            |req, params| Box::pin(crate::rest::purchases::handler::purchases_handlers::get_purchase_handler(req, params)),
        ),
        (
            "POST",
            PURCHASES_LIST_PATH,
            |req, params| Box::pin(crate::rest::purchases::handler::purchases_handlers::list_purchases_handler(req, params)),
        ),
        (
            "POST",
            PURCHASES_CREATE_PATH,
            |req, params| Box::pin(crate::rest::purchases::handler::purchases_handlers::create_purchase_handler(req, params)),
        ),
        (
            "POST",
            PURCHASES_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::purchases::handler::purchases_handlers::update_purchase_handler(req, params)),
        ),
        (
            "POST",
            PURCHASES_DELETE_PATH,
            |req, params| Box::pin(crate::rest::purchases::handler::purchases_handlers::delete_purchase_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }
}