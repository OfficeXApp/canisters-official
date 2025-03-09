// src/rest/contacts/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const CONTACTS_GET_PATH: &str =     genroute!("/contacts/get/{contact_id}");
pub const CONTACTS_LIST_PATH: &str =    genroute!("/contacts/list");
pub const CONTACTS_CREATE_PATH: &str =  genroute!("/contacts/create");
pub const CONTACTS_UPDATE_PATH: &str =  genroute!("/contacts/update");
pub const CONTACTS_DELETE_PATH: &str =  genroute!("/contacts/delete");
pub const CONTACTS_REDEEM_PATH: &str =  genroute!("/contacts/redeem");

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
            CONTACTS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::create_contact_handler(req, params)),
        ),
        (
            "POST",
            CONTACTS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::update_contact_handler(req, params)),
        ),
        (
            "POST",
            CONTACTS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::delete_contact_handler(req, params)),
        ),
        (
            "POST",
            CONTACTS_REDEEM_PATH,
            |req, params| Box::pin(crate::rest::contacts::handler::contacts_handlers::redeem_contact_handler(req, params)),
        )
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}