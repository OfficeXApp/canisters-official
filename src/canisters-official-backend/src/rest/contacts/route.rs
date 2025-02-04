// src/rest/contacts/route.rs
use crate::debug_log;
use crate::rest::router;
use crate::types::RouteHandler;

pub const CONTACTS_GET_PATH: &str = "/contacts/get/{contact_id}";
pub const CONTACTS_LIST_PATH: &str = "/contacts/list";
pub const CONTACTS_UPSERT_PATH: &str = "/contacts/upsert";
pub const CONTACTS_DELETE_PATH: &str = "/contacts/delete";

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            CONTACTS_GET_PATH,
            crate::rest::contacts::handler::contacts_handlers::get_contact_handler,
        ),
        (
            "POST",
            CONTACTS_LIST_PATH,
            crate::rest::contacts::handler::contacts_handlers::list_contacts_handler,
        ),
        (
            "POST",
            CONTACTS_UPSERT_PATH,
            crate::rest::contacts::handler::contacts_handlers::upsert_contact_handler,
        ),
        (
            "POST",
            CONTACTS_DELETE_PATH,
            crate::rest::contacts::handler::contacts_handlers::delete_contact_handler,
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}