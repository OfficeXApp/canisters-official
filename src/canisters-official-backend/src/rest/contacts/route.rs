// src/rest/contacts/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::types::RouteHandler;

pub const CONTACTS_GET_PATH: &str =     genroute!("/contacts/get/{contact_id}");
pub const CONTACTS_LIST_PATH: &str =    genroute!("/contacts/list");
pub const CONTACTS_UPSERT_PATH: &str =  genroute!("/contacts/upsert");
pub const CONTACTS_DELETE_PATH: &str =  genroute!("/contacts/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            CONTACTS_GET_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::get_contact_handler(req, params)),
        ),
        (
            "POST",
            CONTACTS_LIST_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::list_contacts_handler(req, params)),
        ),
        (
            "POST",
            CONTACTS_UPSERT_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::upsert_contact_handler(req, params)),
        ),
        (
            "POST",
            CONTACTS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::delete_contact_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}