// src/rest/teams/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const TEAMS_GET_PATH: &str =        genroute!("/teams/get/{team_id}");
pub const TEAMS_LIST_PATH: &str =       genroute!("/teams/list");
pub const TEAMS_CREATE_PATH: &str =     genroute!("/teams/create");
pub const TEAMS_UPDATE_PATH: &str =     genroute!("/teams/update");
pub const TEAMS_DELETE_PATH: &str =     genroute!("/teams/delete");
pub const TEAMS_VALIDATE_PATH: &str =   genroute!("/teams/validate");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            TEAMS_GET_PATH,
            |req, params| Box::pin(crate::rest::teams::handler::teams_handlers::get_team_handler(req, params)),
        ),
        (
            "POST",
            TEAMS_LIST_PATH,
            |req, params| Box::pin(crate::rest::teams::handler::teams_handlers::list_teams_handler(req, params)),
        ),
        (
            "POST",
            TEAMS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::teams::handler::teams_handlers::create_team_handler(req, params)),
        ),
        (
            "POST",
            TEAMS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::teams::handler::teams_handlers::update_team_handler(req, params)),
        ),
        (
            "POST",
            TEAMS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::teams::handler::teams_handlers::delete_team_handler(req, params)),
        ),
        (
            "POST",
            TEAMS_VALIDATE_PATH,
            |req, params| Box::pin(crate::rest::teams::handler::teams_handlers::validate_team_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}