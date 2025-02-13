// src/rest/permissions/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const PERMISSIONS_GET_PATH: &str = "/permissions/directory/get/{directory_permission_id}";
pub const PERMISSIONS_UPSERT_PATH: &str = "/permissions/directory/upsert";
pub const PERMISSIONS_DELETE_PATH: &str = "/permissions/directory/delete";
pub const PERMISSIONS_CHECK_PATH: &str = "/permissions/directory/check";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            PERMISSIONS_GET_PATH,
            crate::rest::permissions::handler::permissions_handlers::get_permissions_handler,
        ),
        (
            "POST",
            PERMISSIONS_UPSERT_PATH, 
            crate::rest::permissions::handler::permissions_handlers::upsert_permissions_handler,
        ),
        (
            "POST",
            PERMISSIONS_DELETE_PATH,
            crate::rest::permissions::handler::permissions_handlers::delete_permissions_handler,
        ),
        (
            "POST", 
            PERMISSIONS_CHECK_PATH,
            crate::rest::permissions::handler::permissions_handlers::check_permissions_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}