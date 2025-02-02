// src/rest/webhooks/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const WEBHOOKS_GET_PATH: &str = "/webhooks/get/{id}";
pub const WEBHOOKS_LIST_PATH: &str = "/webhooks/list";
pub const WEBHOOKS_UPSERT_PATH: &str = "/webhooks/upsert";
pub const WEBHOOKS_DELETE_PATH: &str = "/webhooks/delete";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            WEBHOOKS_GET_PATH,
            crate::rest::webhooks::handler::webhooks_handlers::get_webhook_handler,
        ),
        (
            "POST",
            WEBHOOKS_LIST_PATH,
            crate::rest::webhooks::handler::webhooks_handlers::list_webhooks_handler,
        ),
        (
            "POST",
            WEBHOOKS_UPSERT_PATH,
            crate::rest::webhooks::handler::webhooks_handlers::create_webhook_handler,
        ),
        (
            "POST",
            WEBHOOKS_DELETE_PATH,
            crate::rest::webhooks::handler::webhooks_handlers::delete_webhook_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}