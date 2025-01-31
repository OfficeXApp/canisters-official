// src/rest/templates/handler.rs
pub mod templates_handlers {
    use crate::{
        certifications::{self, get_certified_response, certify_list_templates_response},
        types::*,
        NEXT_TEMPLATE_ID,
        TEMPLATE_ITEMS,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub fn query_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        certifications::get_certified_response(request)
    }

    pub fn create_template_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let req_body: CreateTemplateRequest = json_decode(req.body());

        let id = NEXT_TEMPLATE_ID.with_borrow_mut(|f| {
            let id = *f;
            *f += 1;
            id
        });

        let template_item = TEMPLATE_ITEMS.with_borrow_mut(|items| {
            let template_item = TemplateItem {
                id,
                title: req_body.title,
                completed: false,
            };

            items.insert(id, template_item.clone());
            template_item
        });

        certify_list_templates_response();

        let body = CreateTemplateResponse::ok(&template_item).encode();
        create_response(StatusCode::CREATED, body)
    }

    pub fn update_template_item_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let req_body: UpdateTemplateRequest = json_decode(req.body());
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        TEMPLATE_ITEMS.with_borrow_mut(|items| {
            let item = items.get_mut(&id).unwrap();

            if let Some(title) = req_body.title {
                item.title = title;
            }

            if let Some(completed) = req_body.completed {
                item.completed = completed;
            }
        });

        certify_list_templates_response();

        let body = UpdateTemplateResponse::ok(&()).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn delete_template_handler(_req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        TEMPLATE_ITEMS.with_borrow_mut(|items| {
            items.remove(&id);
        });

        certify_list_templates_response();

        let body = DeleteTemplateResponse::ok(&()).encode();
        create_response(StatusCode::NO_CONTENT, body)
    }

    pub fn upgrade_to_update_call_handler(
        _http_request: &HttpRequest,
        _params: &Params,
    ) -> HttpResponse<'static> {
        HttpResponse::builder().with_upgrade(true).build()
    }

    pub fn no_update_call_handler(_http_request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        create_response(StatusCode::BAD_REQUEST, vec![])
    }

    fn json_decode<T>(value: &[u8]) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_slice(value).expect("Failed to deserialize value")
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
}