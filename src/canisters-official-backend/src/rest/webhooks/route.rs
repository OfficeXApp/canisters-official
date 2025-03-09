// src/rest/webhooks/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const WEBHOOKS_GET_PATH: &str =     genroute!("/webhooks/get/{webhook_id}");
pub const WEBHOOKS_LIST_PATH: &str =    genroute!("/webhooks/list");
pub const WEBHOOKS_CREATE_PATH: &str =  genroute!("/webhooks/create");
pub const WEBHOOKS_UPDATE_PATH: &str =  genroute!("/webhooks/update");
pub const WEBHOOKS_DELETE_PATH: &str =  genroute!("/webhooks/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            WEBHOOKS_GET_PATH,
            |req, params| Box::pin(crate::rest::webhooks::handler::webhooks_handlers::get_webhook_handler(req, params)),
        ),
        (
            "POST",
            WEBHOOKS_LIST_PATH,
            |req, params| Box::pin(crate::rest::webhooks::handler::webhooks_handlers::list_webhooks_handler(req, params)),
        ),
        (
            "POST",
            WEBHOOKS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::webhooks::handler::webhooks_handlers::create_webhook_handler(req, params)),
        ),
        (
            "POST",
            WEBHOOKS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::webhooks::handler::webhooks_handlers::update_webhook_handler(req, params)),
        ),
        (
            "POST",
            WEBHOOKS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::webhooks::handler::webhooks_handlers::delete_webhook_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}