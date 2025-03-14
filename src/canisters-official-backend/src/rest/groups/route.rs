// src/rest/groups/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const GROUPS_GET_PATH: &str =        genroute!("/groups/get/{group_id}");
pub const GROUPS_LIST_PATH: &str =       genroute!("/groups/list");
pub const GROUPS_CREATE_PATH: &str =     genroute!("/groups/create");
pub const GROUPS_UPDATE_PATH: &str =     genroute!("/groups/update");
pub const GROUPS_DELETE_PATH: &str =     genroute!("/groups/delete");
pub const GROUPS_VALIDATE_PATH: &str =   genroute!("/groups/validate");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            GROUPS_GET_PATH,
            |req, params| Box::pin(crate::rest::groups::handler::groups_handlers::get_group_handler(req, params)),
        ),
        (
            "POST",
            GROUPS_LIST_PATH,
            |req, params| Box::pin(crate::rest::groups::handler::groups_handlers::list_groups_handler(req, params)),
        ),
        (
            "POST",
            GROUPS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::groups::handler::groups_handlers::create_group_handler(req, params)),
        ),
        (
            "POST",
            GROUPS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::groups::handler::groups_handlers::update_group_handler(req, params)),
        ),
        (
            "POST",
            GROUPS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::groups::handler::groups_handlers::delete_group_handler(req, params)),
        ),
        (
            "POST",
            GROUPS_VALIDATE_PATH,
            |req, params| Box::pin(crate::rest::groups::handler::groups_handlers::validate_group_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}