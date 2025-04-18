// src/rest/directory/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const DIRECTORYS_LIST_PATH: &str =      genroute!("/directory/list");
pub const DIRECTORYS_ACTION_PATH: &str =    genroute!("/directory/action");
pub const UPLOAD_CHUNK_PATH: &str =         genroute!("/directory/raw_upload/chunk");
pub const COMPLETE_UPLOAD_PATH: &str =      genroute!("/directory/raw_upload/complete");
pub const RAW_DOWNLOAD_META_PATH: &str =    genroute!("/directory/raw_download/meta");
pub const RAW_DOWNLOAD_CHUNK_PATH: &str =   genroute!("/directory/raw_download/chunk");
pub const RAW_URL_PROXY_PATH: &str =        genroute!("/directory/asset/{file_id_with_extension}"); // for proxying raw urls 302 redirect to temp presigned s3 urls


type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "POST",
            DIRECTORYS_LIST_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::list_directorys_handler(req, params)),
        ),
        (
            "POST",
            DIRECTORYS_ACTION_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::action_directory_handler(req, params)),
        ),
        (
            "POST",
            UPLOAD_CHUNK_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::handle_upload_chunk(req, params)),
        ),
        (
            "POST",
            COMPLETE_UPLOAD_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::handle_complete_upload(req, params)),
        ),
        (
            "GET",
            RAW_DOWNLOAD_META_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::download_file_metadata_handler(req, params)),
        ),
        (
            "GET",
            RAW_DOWNLOAD_CHUNK_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::download_file_chunk_handler(req, params)),
        ),
        (
            "GET",
            RAW_URL_PROXY_PATH,
            |req, params| Box::pin(crate::rest::directory::handler::directorys_handlers::get_raw_url_proxy_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}