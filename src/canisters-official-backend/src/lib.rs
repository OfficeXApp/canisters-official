// src/lib.rs
use ic_cdk::*;
use ic_http_certification::{HttpRequest, HttpResponse};
use core::state::apikeys::state::state::init_default_admin_apikey;
use std::{cell::RefCell, collections::HashMap};

mod logger;
mod types;
mod rest;
mod core;
use rest::{router};

#[ic_cdk_macros::init]
fn init() {
    debug_log!("Initializing canister...");
    router::init_routes();
    init_default_admin_apikey();
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
