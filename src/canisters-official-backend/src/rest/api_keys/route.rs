// src/rest/api_keys/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const APIKEYS_GET_PATH: &str = "/api_keys/get/{api_key_id}";
pub const APIKEYS_LIST_PATH: &str = "/api_keys/list/{user_id}";
pub const APIKEYS_UPSERT_PATH: &str = "/api_keys/upsert";
pub const APIKEYS_DELETE_PATH: &str = "/api_keys/delete";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            APIKEYS_GET_PATH,
            crate::rest::api_keys::handler::apikeys_handlers::get_apikey_handler,
        ),
        (
            "POST",
            APIKEYS_LIST_PATH,
            crate::rest::api_keys::handler::apikeys_handlers::list_apikeys_handler,
        ),
        (
            "POST",
            APIKEYS_UPSERT_PATH,
            crate::rest::api_keys::handler::apikeys_handlers::upsert_apikey_handler,
        ),
        (
            "POST",
            APIKEYS_DELETE_PATH,
            crate::rest::api_keys::handler::apikeys_handlers::delete_apikey_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}