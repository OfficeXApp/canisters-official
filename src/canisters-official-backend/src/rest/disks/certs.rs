// src/rest/disks/certs.rs


#[derive(Debug, Clone)]
struct CertifiedHttpResponse<'a> {
    response: HttpResponse<'a>,
    certification: HttpCertification,
}

use crate::rest::disks::route::DISKS_LIST_PATH;


use ic_http_certification::{
    DefaultCelBuilder, DefaultFullCelExpression, DefaultResponseCertification, DefaultResponseOnlyCelExpression,
    HttpCertification, HttpCertificationPath, HttpCertificationTree,
    HttpResponse, 
};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::collections::HashMap;



thread_local! {
    static HTTP_TREE: RefCell<HttpCertificationTree> = RefCell::new(HttpCertificationTree::default());
    static FALLBACK_RESPONSES: RefCell<HashMap<String, CertifiedHttpResponse<'static>>> = RefCell::new(HashMap::new());
    static RESPONSES: RefCell<HashMap<(String, String), CertifiedHttpResponse<'static>>> = RefCell::new(HashMap::new());
}

const NOT_FOUND_PATH: &str = "";


lazy_static! {
    pub static ref DISKS_TREE_PATH: HttpCertificationPath<'static> = HttpCertificationPath::exact(DISKS_LIST_PATH);
    static ref NOT_FOUND_TREE_PATH: HttpCertificationPath<'static> = HttpCertificationPath::wildcard(NOT_FOUND_PATH);

    static ref DISKS_CEL_EXPR_DEF: DefaultFullCelExpression<'static> = DefaultCelBuilder::full_certification()
        .with_request_headers(vec![])
        .with_request_query_parameters(vec![])
        .with_response_certification(DefaultResponseCertification::response_header_exclusions(vec![]))
        .build();
    static ref DISKS_CEL_EXPR: String = DISKS_CEL_EXPR_DEF.to_string();

    static ref NOT_FOUND_CEL_EXPR_DEF: DefaultResponseOnlyCelExpression<'static> = DefaultCelBuilder::response_only_certification()
        .with_response_certification(DefaultResponseCertification::response_header_exclusions(vec![]))
        .build();
    static ref NOT_FOUND_CEL_EXPR: String = NOT_FOUND_CEL_EXPR_DEF.to_string();
}
