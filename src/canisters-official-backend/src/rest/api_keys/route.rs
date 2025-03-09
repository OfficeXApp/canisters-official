// src/rest/api_keys/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


// ROUTE_PREFIX
pub const APIKEYS_GET_PATH: &str =      genroute!("/api_keys/get/{api_key_id}");
pub const APIKEYS_LIST_PATH: &str =     genroute!("/api_keys/list/{user_id}");
pub const APIKEYS_CREATE_PATH: &str =   genroute!("/api_keys/create");
pub const APIKEYS_UPDATE_PATH: &str =   genroute!("/api_keys/update");
pub const APIKEYS_DELETE_PATH: &str =   genroute!("/api_keys/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            APIKEYS_GET_PATH,
            |req, params| Box::pin(crate::rest::api_keys::handler::apikeys_handlers::get_apikey_handler(req, params)),
        ),
        (
            "POST",
            APIKEYS_LIST_PATH,
            |req, params| Box::pin(crate::rest::api_keys::handler::apikeys_handlers::list_apikeys_handler(req, params)),
        ),
        (
            "POST",
            APIKEYS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::api_keys::handler::apikeys_handlers::create_apikey_handler(req, params)),
        ),
        (
            "POST",
            APIKEYS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::api_keys::handler::apikeys_handlers::update_apikey_handler(req, params)),
        ),
        (
            "POST",
            APIKEYS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::api_keys::handler::apikeys_handlers::delete_apikey_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}