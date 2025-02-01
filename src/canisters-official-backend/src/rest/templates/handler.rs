// src/rest/templates/handler.rs
pub mod templates_handlers {
    use crate::{
        certifications::{self, get_certified_response}, debug_log, rest::templates::certs::{certify_response, TEMPLATES_TREE_PATH}, types::*, NEXT_TEMPLATE_ID, TEMPLATE_ITEMS
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub fn query_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        debug_log!("query_handler called for path: {}", request.get_path().unwrap_or_default());
        let path = request.get_path().expect("Failed to get request path");
    
        match path {
            TEMPLATES_LIST_PATH => handle_list_templates(request),
            _ => certifications::get_certified_response(request)
        }
    }

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
        // Add other filter fields as needed
    }

    fn handle_list_templates(request: &HttpRequest) -> HttpResponse<'static> {
        debug_log!("Handling list templates request");
        
        // Parse query parameters
        let query_params = request.get_query()
            .ok()
            .flatten()  // Convert Option<Option<String>> to Option<String>
            .and_then(|q| serde_urlencoded::from_str::<ListQueryParams>(&q).ok())
            .unwrap_or_default();
        
        // Get and filter items
        let items = TEMPLATE_ITEMS.with_borrow(|items| {
            items.iter()
                .filter(|(_id, item)| {
                    // Apply filters based on query params
                    if let Some(title) = &query_params.title {
                        if !item.title.contains(title) {
                            return false;
                        }
                    }
                    if let Some(completed) = query_params.completed {
                        if item.completed != completed {
                            return false;
                        }
                    }
                    true
                })
                .map(|(_id, item)| item.clone())
                .collect::<Vec<_>>()
        });
    
        let body = ListTemplatesResponse::ok(&items).encode();
        let mut response = create_response(StatusCode::OK, body);
        certify_response(request.clone(), &mut response, &TEMPLATES_TREE_PATH);
    
        response
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

        

        let body = UpdateTemplateResponse::ok(&()).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn delete_template_handler(_req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        TEMPLATE_ITEMS.with_borrow_mut(|items| {
            items.remove(&id);
        });

        

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