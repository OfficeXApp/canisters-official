// src/rest/directorys/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const DIRECTORYS_SEARCH_PATH: &str = "/directory/search";
pub const DIRECTORYS_LIST_PATH: &str = "/directory/list";
pub const DIRECTORYS_ACTION_PATH: &str = "/directory/action";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "POST",
            DIRECTORYS_SEARCH_PATH,
            crate::rest::directory::handler::directorys_handlers::search_directory_handler,
        ),
        (
            "POST",
            DIRECTORYS_LIST_PATH,
            crate::rest::directory::handler::directorys_handlers::list_directorys_handler,
        ),
        (
            "POST",
            DIRECTORYS_ACTION_PATH,
            crate::rest::directory::handler::directorys_handlers::action_directory_handler,
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}