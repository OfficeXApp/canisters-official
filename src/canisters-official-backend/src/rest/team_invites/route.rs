// src/rest/team_invites/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const TEAM_INVITES_GET_PATH: &str = "/team_invites/get/{id}";
pub const TEAM_INVITES_LIST_PATH: &str = "/team_invites/list";
pub const TEAM_INVITES_UPSERT_PATH: &str = "/team_invites/upsert";
pub const TEAM_INVITES_DELETE_PATH: &str = "/team_invites/delete";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            TEAM_INVITES_GET_PATH,
            crate::rest::team_invites::handler::team_invites_handlers::get_team_invite_handler,
        ),
        (
            "POST",
            TEAM_INVITES_LIST_PATH,
            crate::rest::team_invites::handler::team_invites_handlers::list_team_invites_handler,
        ),
        (
            "POST",
            TEAM_INVITES_UPSERT_PATH,
            crate::rest::team_invites::handler::team_invites_handlers::upsert_team_invite_handler,
        ),
        (
            "POST",
            TEAM_INVITES_DELETE_PATH,
            crate::rest::team_invites::handler::team_invites_handlers::delete_team_invite_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}