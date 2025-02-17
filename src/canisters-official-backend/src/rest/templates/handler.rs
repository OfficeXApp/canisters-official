// src/rest/templates/handler.rs


pub mod templates_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::templates::{state::state::TEMPLATE_ITEMS, types::TemplateID}, types::IDPrefix}, debug_log, rest::templates::types::{CreateTemplateRequest, CreateTemplateResponse, DeleteTemplateRequest, DeleteTemplateResponse, DeletedTemplateData, ErrorResponse, GetTemplateResponse, ListTemplatesResponse, UpdateTemplateRequest, UpdateTemplateResponse}
        
    };
    use crate::core::state::templates::{
        types::TemplateItem,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_template_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let id = TemplateID(params.get("id").unwrap().to_string());

        let item = TEMPLATE_ITEMS.with_borrow(|items| {
            items.get(&id).cloned()
        });

        match item {
            Some(item) => {
                let body = GetTemplateResponse::ok(&item).encode();
                create_response(StatusCode::OK, body)
            }
            None => {
                let body = ErrorResponse::not_found().encode();
                create_response(StatusCode::NOT_FOUND, body)
            }
        }
    }

    pub async fn list_templates_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Handling list templates request");
        
        let query_params = request.get_query()
            .ok()
            .flatten()
            .and_then(|q| serde_urlencoded::from_str::<ListQueryParams>(&q).ok())
            .unwrap_or_default();
        
        let items = TEMPLATE_ITEMS.with_borrow(|items| {
            items.iter()
                .filter(|(_id, item)| {
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
        create_response(StatusCode::OK, body)
    }

    pub async fn upsert_template_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let req_body: CreateTemplateRequest = json_decode(request.body());

        let id = TemplateID(generate_unique_id(IDPrefix::User, ""));

        let template_item = TEMPLATE_ITEMS.with_borrow_mut(|items| {
            let template_item = TemplateItem {
                id: id.clone(),
                title: req_body.title,
                completed: false,
            };

            items.insert(id.clone(), template_item.clone());
            template_item
        });

        let body = CreateTemplateResponse::ok(&template_item).encode();
        create_response(StatusCode::CREATED, body)
    }

    pub async fn delete_template_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let req_body: DeleteTemplateRequest = json_decode(request.body());

        let id = req_body.id.clone();

        TEMPLATE_ITEMS.with_borrow_mut(|items| {
            items.remove(&id);
        });

        let deleted_data = DeletedTemplateData {
            id: req_body.id,
            deleted: true,
        };

        let body = DeleteTemplateResponse::ok(&deleted_data).encode();
        create_response(StatusCode::OK, body)
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