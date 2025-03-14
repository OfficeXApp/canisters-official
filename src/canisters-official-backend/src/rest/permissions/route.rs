// src/rest/permissions/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;


pub const DIRECTORY_PERMISSIONS_GET_PATH: &str =    genroute!("/permissions/directory/get/{directory_permission_id}");
pub const DIRECTORY_PERMISSIONS_CREATE_PATH: &str = genroute!("/permissions/directory/create");
pub const DIRECTORY_PERMISSIONS_UPDATE_PATH: &str = genroute!("/permissions/directory/update");
pub const DIRECTORY_PERMISSIONS_DELETE_PATH: &str = genroute!("/permissions/directory/delete");
pub const DIRECTORY_PERMISSIONS_CHECK_PATH: &str =  genroute!("/permissions/directory/check");
pub const DIRECTORY_PERMISSIONS_REDEEM_PATH: &str = genroute!("/permissions/directory/redeem");

pub const SYSTEM_PERMISSIONS_GET_PATH: &str =       genroute!("/permissions/system/get/{system_permission_id}");
pub const SYSTEM_PERMISSIONS_LIST_PATH: &str =      genroute!("/permissions/system/list");
pub const SYSTEM_PERMISSIONS_CREATE_PATH: &str =    genroute!("/permissions/system/create");
pub const SYSTEM_PERMISSIONS_UPDATE_PATH: &str =    genroute!("/permissions/system/update");
pub const SYSTEM_PERMISSIONS_DELETE_PATH: &str =    genroute!("/permissions/system/delete");
pub const SYSTEM_PERMISSIONS_CHECK_PATH: &str =     genroute!("/permissions/system/check");
pub const SYSTEM_PERMISSIONS_REDEEM_PATH: &str =    genroute!("/permissions/system/redeem");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            DIRECTORY_PERMISSIONS_GET_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::get_directory_permissions_handler(req, params)),
        ),
        (
            "POST",
            DIRECTORY_PERMISSIONS_CREATE_PATH, 
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::create_directory_permissions_handler(req, params)),
        ),
        (
            "POST",
            DIRECTORY_PERMISSIONS_UPDATE_PATH, 
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::update_directory_permissions_handler(req, params)),
        ),
        (
            "POST",
            DIRECTORY_PERMISSIONS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::delete_directory_permissions_handler(req, params)),
        ),
        (
            "POST", 
            DIRECTORY_PERMISSIONS_CHECK_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::check_directory_permissions_handler(req, params)),
        ),
        (
            "POST", 
            DIRECTORY_PERMISSIONS_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::redeem_directory_permissions_handler(req, params)),
        ),
        // 
        (
            "GET",
            SYSTEM_PERMISSIONS_GET_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::get_system_permissions_handler(req, params)),
        ),
        (
            "POST",
            SYSTEM_PERMISSIONS_LIST_PATH, 
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::list_system_permissions_handler(req, params)),
        ),
        (
            "POST",
            SYSTEM_PERMISSIONS_CREATE_PATH, 
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::create_system_permissions_handler(req, params)),
        ),
        (
            "POST",
            SYSTEM_PERMISSIONS_UPDATE_PATH, 
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::update_system_permissions_handler(req, params)),
        ),
        (
            "POST",
            SYSTEM_PERMISSIONS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::delete_system_permissions_handler(req, params)),
        ),
        (
            "POST", 
            SYSTEM_PERMISSIONS_CHECK_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::check_system_permissions_handler(req, params)),
        ),
        (
            "POST", 
            SYSTEM_PERMISSIONS_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::permissions::handler::permissions_handlers::redeem_system_permissions_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}