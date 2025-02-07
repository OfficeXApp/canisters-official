// src/rest/directorys/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const DIRECTORYS_SEARCH_PATH: &str = "/directory/search";
pub const DIRECTORYS_LIST_PATH: &str = "/directory/list";
pub const DIRECTORYS_ACTION_PATH: &str = "/directory/action";
pub const UPLOAD_CHUNK_PATH: &str = "/directory/raw_upload/chunk";
pub const COMPLETE_UPLOAD_PATH: &str = "/directory/raw_upload/complete";
pub const RAW_DOWNLOAD_META_PATH: &str = "/directory/raw_download/meta";
pub const RAW_DOWNLOAD_CHUNK_PATH: &str = "/directory/raw_download/chunk";

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
        (
            "POST",
            UPLOAD_CHUNK_PATH,
            crate::rest::directory::handler::directorys_handlers::handle_upload_chunk,
        ),
        (
            "POST",
            COMPLETE_UPLOAD_PATH,
            crate::rest::directory::handler::directorys_handlers::handle_complete_upload,
        ),
        (
            "GET",
            RAW_DOWNLOAD_META_PATH,
            crate::rest::directory::handler::directorys_handlers::download_file_metadata_handler,
        ),
        (
            "GET",
            RAW_DOWNLOAD_CHUNK_PATH,
            crate::rest::directory::handler::directorys_handlers::download_file_chunk_handler,
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}