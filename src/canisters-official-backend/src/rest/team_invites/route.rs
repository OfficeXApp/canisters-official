// src/rest/team_invites/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const TEAM_INVITES_GET_PATH: &str =     genroute!("/teams/invites/get/{invite_id}");
pub const TEAM_INVITES_LIST_PATH: &str =    genroute!("/teams/invites/list");
pub const TEAM_INVITES_CREATE_PATH: &str =  genroute!("/teams/invites/create");
pub const TEAM_INVITES_UPDATE_PATH: &str =  genroute!("/teams/invites/update");
pub const TEAM_INVITES_DELETE_PATH: &str =  genroute!("/teams/invites/delete");
pub const TEAM_INVITES_REDEEM_PATH: &str =  genroute!("/teams/invites/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            TEAM_INVITES_GET_PATH,
            |req, params| Box::pin(crate::rest::team_invites::handler::team_invites_handlers::get_team_invite_handler(req, params)),
        ),
        (
            "POST",
            TEAM_INVITES_LIST_PATH,
            |req, params| Box::pin(crate::rest::team_invites::handler::team_invites_handlers::list_team_invites_handler(req, params)),
        ),
        (
            "POST",
            TEAM_INVITES_CREATE_PATH,
            |req, params| Box::pin(crate::rest::team_invites::handler::team_invites_handlers::create_team_invite_handler(req, params)),
        ),
        (
            "POST",
            TEAM_INVITES_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::team_invites::handler::team_invites_handlers::update_team_invite_handler(req, params)),
        ),
        (
            "POST",
            TEAM_INVITES_DELETE_PATH,
            |req, params| Box::pin(crate::rest::team_invites::handler::team_invites_handlers::delete_team_invite_handler(req, params)),
        ),
        (
            "POST",
            TEAM_INVITES_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::team_invites::handler::team_invites_handlers::redeem_team_invite_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}