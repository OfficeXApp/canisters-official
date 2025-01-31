// http/certifications.rs
use crate::{routes::*, types::*, TODO_ITEMS};
use ic_cdk::api::{data_certificate, set_certified_data};
use ic_http_certification::{*, utils::add_v2_certificate_header};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static HTTP_TREE: RefCell<HttpCertificationTree> = RefCell::new(HttpCertificationTree::default());
    static FALLBACK_RESPONSES: RefCell<HashMap<String, CertifiedHttpResponse<'static>>> = RefCell::new(HashMap::new());
    static RESPONSES: RefCell<HashMap<(String, String), CertifiedHttpResponse<'static>>> = RefCell::new(HashMap::new());
}

const NOT_FOUND_PATH: &str = "";

#[derive(Debug, Clone)]
struct CertifiedHttpResponse<'a> {
    response: HttpResponse<'a>,
    certification: HttpCertification,
}

lazy_static! {
    static ref TODOS_TREE_PATH: HttpCertificationPath<'static> = HttpCertificationPath::exact(TODOS_PATH);
    static ref NOT_FOUND_TREE_PATH: HttpCertificationPath<'static> = HttpCertificationPath::wildcard(NOT_FOUND_PATH);

    static ref TODO_CEL_EXPR_DEF: DefaultFullCelExpression<'static> = DefaultCelBuilder::full_certification()
        .with_request_headers(vec![])
        .with_request_query_parameters(vec![])
        .with_response_certification(DefaultResponseCertification::response_header_exclusions(vec![]))
        .build();
    static ref TODO_CEL_EXPR: String = TODO_CEL_EXPR_DEF.to_string();

    static ref NOT_FOUND_CEL_EXPR_DEF: DefaultResponseOnlyCelExpression<'static> = DefaultCelBuilder::response_only_certification()
        .with_response_certification(DefaultResponseCertification::response_header_exclusions(vec![]))
        .build();
    static ref NOT_FOUND_CEL_EXPR: String = NOT_FOUND_CEL_EXPR_DEF.to_string();
}

pub fn init_certifications() {
    certify_list_todos_response();
    certify_not_allowed_todo_responses();
    certify_not_found_response();
}

pub fn certify_list_todos_response() {
    let request = HttpRequest::get(TODOS_PATH).build();

    let body = TODO_ITEMS.with_borrow(|items| {
        ListTodosResponse::ok(
            &items
                .iter()
                .map(|(_id, item)| item.clone())
                .collect::<Vec<_>>(),
        )
        .encode()
    });
    let mut response = create_response(StatusCode::OK, body);

    certify_response(request, &mut response, &TODOS_TREE_PATH);
}

fn certify_not_allowed_todo_responses() {
    [
        Method::HEAD,
        Method::PUT,
        Method::PATCH,
        Method::OPTIONS,
        Method::TRACE,
        Method::CONNECT,
    ]
    .into_iter()
    .for_each(|method| {
        let request = HttpRequest::builder()
            .with_method(method)
            .with_url(TODOS_PATH)
            .build();

        let body = ErrorResponse::not_allowed().encode();
        let mut response = create_response(StatusCode::METHOD_NOT_ALLOWED, body);

        certify_response(request, &mut response, &TODOS_TREE_PATH);
    });
}

fn certify_not_found_response() {
    let body = ErrorResponse::not_found().encode();
    let mut response = create_response(StatusCode::NOT_FOUND, body);

    let tree_path = HttpCertificationPath::wildcard(NOT_FOUND_PATH);
    response.add_header((
        CERTIFICATE_EXPRESSION_HEADER_NAME.to_string(),
        NOT_FOUND_CEL_EXPR.clone(),
    ));

    let certification = HttpCertification::response_only(&NOT_FOUND_CEL_EXPR_DEF, &response, None).unwrap();

    FALLBACK_RESPONSES.with_borrow_mut(|responses| {
        responses.insert(
            NOT_FOUND_PATH.to_string(),
            CertifiedHttpResponse {
                response,
                certification,
            },
        );
    });

    HTTP_TREE.with_borrow_mut(|http_tree| {
        http_tree.insert(&HttpCertificationTreeEntry::new(tree_path, &certification));
        set_certified_data(&http_tree.root_hash());
    });
}

pub fn get_certified_response(request: &HttpRequest) -> HttpResponse<'static> {
    let request_path = request.get_path().expect("Failed to get req path");

    let (tree_path, certified_response) = RESPONSES
        .with_borrow(|responses| {
            responses
                .get(&(request.method().to_string(), request_path.clone()))
                .map(|response| (HttpCertificationPath::exact(&request_path), response.clone()))
        })
        .unwrap_or_else(|| {
            FALLBACK_RESPONSES.with_borrow(|fallback_responses| {
                fallback_responses
                    .get(NOT_FOUND_PATH)
                    .clone()
                    .map(|response| (NOT_FOUND_TREE_PATH.to_owned(), response.clone()))
                    .unwrap()
            })
        });

    let mut response = certified_response.response;

    HTTP_TREE.with_borrow(|http_tree| {
        add_v2_certificate_header(
            &data_certificate().expect("No data certificate available"),
            &mut response,
            &http_tree
                .witness(
                    &HttpCertificationTreeEntry::new(&tree_path, certified_response.certification),
                    &request_path,
                )
                .unwrap(),
            &tree_path.to_expr_path(),
        );
    });

    response
}

fn certify_response(
    request: HttpRequest,
    response: &mut HttpResponse<'static>,
    tree_path: &HttpCertificationPath,
) {
    let request_path = request.get_path().unwrap();

    let existing_response = RESPONSES.with_borrow_mut(|responses| {
        responses.remove(&(request.method().to_string(), request_path.clone()))
    });

    if let Some(existing_response) = existing_response {
        HTTP_TREE.with_borrow_mut(|http_tree| {
            http_tree.delete(&HttpCertificationTreeEntry::new(
                tree_path.clone(),
                &existing_response.certification,
            ));
        })
    }

    response.add_header((
        CERTIFICATE_EXPRESSION_HEADER_NAME.to_string(),
        TODO_CEL_EXPR.clone(),
    ));

    let certification =
        HttpCertification::full(&TODO_CEL_EXPR_DEF, &request, &response, None).unwrap();

    RESPONSES.with_borrow_mut(|responses| {
        responses.insert(
            (request.method().to_string(), request_path),
            CertifiedHttpResponse {
                response: response.clone(),
                certification: certification.clone(),
            },
        );
    });

    HTTP_TREE.with_borrow_mut(|http_tree| {
        http_tree.insert(&HttpCertificationTreeEntry::new(tree_path, &certification));
        set_certified_data(&http_tree.root_hash());
    });
}

fn create_response(status_code: StatusCode, body: Vec<u8>) -> HttpResponse<'static> {
    HttpResponse::builder()
        .with_status_code(status_code)
        .with_headers(vec![
            ("content-type".to_string(), "application/json".to_string()),
            (
                "strict-transport-security".to_string(),
                "max-age=31536000; includeSubDomains".to_string(),
            ),
            ("x-content-type-options".to_string(), "nosniff".to_string()),
            ("referrer-policy".to_string(), "no-referrer".to_string()),
            (
                "cache-control".to_string(),
                "no-store, max-age=0".to_string(),
            ),
            ("pragma".to_string(), "no-cache".to_string()),
        ])
        .with_body(body)
        .build()
}