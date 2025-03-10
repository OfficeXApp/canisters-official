// src/rest/tags/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const TAGS_GET_PATH: &str =         genroute!("/tags/get/{id}");
pub const TAGS_LIST_PATH: &str =        genroute!("/tags/list");
pub const TAGS_CREATE_PATH: &str =      genroute!("/tags/create");
pub const TAGS_UPDATE_PATH: &str =      genroute!("/tags/update");
pub const TAGS_DELETE_PATH: &str =      genroute!("/tags/delete");
pub const TAGS_RESOURCE_PATH: &str =    genroute!("/tags/pin");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            TAGS_GET_PATH,
            |req, params| Box::pin(crate::rest::tags::handler::tags_handlers::get_tag_handler(req, params)),
        ),
        (
            "POST",
            TAGS_LIST_PATH,
            |req, params| Box::pin(crate::rest::tags::handler::tags_handlers::list_tags_handler(req, params)),
        ),
        (
            "POST",
            TAGS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::tags::handler::tags_handlers::create_tag_handler(req, params)),
        ),
        (
            "POST",
            TAGS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::tags::handler::tags_handlers::update_tag_handler(req, params)),
        ),
        (
            "POST",
            TAGS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::tags::handler::tags_handlers::delete_tag_handler(req, params)),
        ),
        (
            "POST",
            TAGS_RESOURCE_PATH,
            |req, params| Box::pin(crate::rest::tags::handler::tags_handlers::tag_pin_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}