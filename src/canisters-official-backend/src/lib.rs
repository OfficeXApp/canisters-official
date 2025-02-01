// src/lib.rs
use ic_cdk::*;
use ic_http_certification::{HttpRequest, HttpResponse};
use std::{cell::RefCell, collections::HashMap};

mod logger;
mod types;
mod rest;
mod state;

use state::{NEXT_TEMPLATE_ID, TEMPLATE_ITEMS};
use rest::{router, templates::types::TemplateItem};

#[init]
fn init() {
    debug_log!("Initializing canister...");
    router::init_routes();
}

#[post_upgrade]
fn post_upgrade() {
    init();
}

#[query]
fn http_request(_req: HttpRequest) -> HttpResponse<'static> {
    // All requests will be upgraded to update calls
    HttpResponse::builder()
        .with_upgrade(true)
        .build()
}

#[update]
fn http_request_update(req: HttpRequest) -> HttpResponse<'static> {
    router::handle_request(req)
}
