// src/rest/group_invites/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const GROUP_INVITES_GET_PATH: &str =     genroute!("/groups/invites/get/{invite_id}");
pub const GROUP_INVITES_LIST_PATH: &str =    genroute!("/groups/invites/list");
pub const GROUP_INVITES_CREATE_PATH: &str =  genroute!("/groups/invites/create");
pub const GROUP_INVITES_UPDATE_PATH: &str =  genroute!("/groups/invites/update");
pub const GROUP_INVITES_DELETE_PATH: &str =  genroute!("/groups/invites/delete");
pub const GROUP_INVITES_REDEEM_PATH: &str =  genroute!("/groups/invites/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            GROUP_INVITES_GET_PATH,
            |req, params| Box::pin(crate::rest::group_invites::handler::group_invites_handlers::get_group_invite_handler(req, params)),
        ),
        (
            "POST",
            GROUP_INVITES_LIST_PATH,
            |req, params| Box::pin(crate::rest::group_invites::handler::group_invites_handlers::list_group_invites_handler(req, params)),
        ),
        (
            "POST",
            GROUP_INVITES_CREATE_PATH,
            |req, params| Box::pin(crate::rest::group_invites::handler::group_invites_handlers::create_group_invite_handler(req, params)),
        ),
        (
            "POST",
            GROUP_INVITES_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::group_invites::handler::group_invites_handlers::update_group_invite_handler(req, params)),
        ),
        (
            "POST",
            GROUP_INVITES_DELETE_PATH,
            |req, params| Box::pin(crate::rest::group_invites::handler::group_invites_handlers::delete_group_invite_handler(req, params)),
        ),
        (
            "POST",
            GROUP_INVITES_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::group_invites::handler::group_invites_handlers::redeem_group_invite_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}