// src/lib.rs
use ic_cdk::*;
use ic_http_certification::{HttpRequest, HttpResponse};
use std::{cell::RefCell, collections::HashMap};

mod types;
mod rest;
mod certifications;

use rest::{router};
use types::*;

thread_local! {
    static NEXT_TODO_ID: RefCell<u32> = RefCell::new(0);
    static TODO_ITEMS: RefCell<HashMap<u32, TodoItem>> = RefCell::new(HashMap::new());
}

#[init]
fn init() {
    certifications::init_certifications();
    router::init_routes();
}

#[post_upgrade]
fn post_upgrade() {
    init();
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse<'static> {
    router::handle_query_request(req)
}

#[update]
fn http_request_update(req: HttpRequest) -> HttpResponse<'static> {
    router::handle_update_request(req)
}