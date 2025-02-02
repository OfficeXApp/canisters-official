// src/rest/webhooks/handler.rs


pub mod webhooks_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::webhooks::{state::state::WEBHOOK_ITEMS, types::WebhookID}}, debug_log, rest::webhooks::types::{CreateWebhookRequest, CreateWebhookResponse, DeleteWebhookRequest, DeleteWebhookResponse, DeletedWebhookData, ErrorResponse, GetWebhookResponse, ListWebhooksResponse, UpdateWebhookRequest, UpdateWebhookResponse}
        
    };
    use crate::core::state::webhooks::{
        types::WebhookItem,
        state::state::WebhookState 
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_webhook_handler(_req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let id = WebhookID(params.get("id").unwrap().to_string());

        let item = WEBHOOK_ITEMS.with_borrow(|items| {
            items.get(&id).cloned()
        });

        match item {
            Some(item) => {
                let body = GetWebhookResponse::ok(&item).encode();
                create_response(StatusCode::OK, body)
            }
            None => {
                let body = ErrorResponse::not_found().encode();
                create_response(StatusCode::NOT_FOUND, body)
            }
        }
    }

    pub fn list_webhooks_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        debug_log!("Handling list webhooks request");
        
        let query_params = request.get_query()
            .ok()
            .flatten()
            .and_then(|q| serde_urlencoded::from_str::<ListQueryParams>(&q).ok())
            .unwrap_or_default();
        
        let items = WEBHOOK_ITEMS.with_borrow(|items| {
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
    
        let body = ListWebhooksResponse::ok(&items).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn create_webhook_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let req_body: CreateWebhookRequest = json_decode(req.body());

        let id = WebhookID(generate_unique_id("webhookID"));

        let webhook_item = WEBHOOK_ITEMS.with_borrow_mut(|items| {
            let webhook_item = WebhookItem {
                id: id.clone(),
                title: req_body.title,
                completed: false,
            };

            items.insert(id.clone(), webhook_item.clone());
            webhook_item
        });

        let body = CreateWebhookResponse::ok(&webhook_item).encode();
        create_response(StatusCode::CREATED, body)
    }

    pub fn update_webhook_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let req_body: UpdateWebhookRequest = json_decode(req.body());
        let id = WebhookID(params.get("id").unwrap().to_string());

        WEBHOOK_ITEMS.with_borrow_mut(|items| {
            let item = items.get_mut(&id).unwrap();

            if let Some(title) = req_body.title {
                item.title = title;
            }

            if let Some(completed) = req_body.completed {
                item.completed = completed;
            }
        });

        let body = UpdateWebhookResponse::ok(&()).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn delete_webhook_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let req_body: DeleteWebhookRequest = json_decode(req.body());

        let id = req_body.id.clone();

        WEBHOOK_ITEMS.with_borrow_mut(|items| {
            items.remove(&id);
        });

        let deleted_data = DeletedWebhookData {
            id: req_body.id,
            deleted: true,
        };

        let body = DeleteWebhookResponse::ok(&deleted_data).encode();
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