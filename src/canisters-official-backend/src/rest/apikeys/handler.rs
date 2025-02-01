// src/rest/apikeys/handler.rs
pub mod apikeys_handlers {
    use crate::{
        debug_log, 
        rest::apikeys::types::{CreateApiKeyRequest, CreateApiKeyResponse, DeleteApiKeyRequest, DeleteApiKeyResponse, DeletedApiKeyData, ErrorResponse, GetApiKeyResponse, ListApiKeysResponse, ApiKeyItem, UpdateApiKeyRequest, UpdateApiKeyResponse},
        state::{NEXT_APIKEY_ID, APIKEY_ITEMS},
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_apikey_handler(_req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        let item = APIKEY_ITEMS.with_borrow(|items| {
            items.get(&id).cloned()
        });

        match item {
            Some(item) => {
                let body = GetApiKeyResponse::ok(&item).encode();
                create_response(StatusCode::OK, body)
            }
            None => {
                let body = ErrorResponse::not_found().encode();
                create_response(StatusCode::NOT_FOUND, body)
            }
        }
    }

    pub fn list_apikeys_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        debug_log!("Handling list apikeys request");
        
        let query_params = request.get_query()
            .ok()
            .flatten()
            .and_then(|q| serde_urlencoded::from_str::<ListQueryParams>(&q).ok())
            .unwrap_or_default();
        
        let items = APIKEY_ITEMS.with_borrow(|items| {
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
    
        let body = ListApiKeysResponse::ok(&items).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn create_apikey_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let req_body: CreateApiKeyRequest = json_decode(req.body());

        let id = NEXT_APIKEY_ID.with_borrow_mut(|f| {
            let id = *f;
            *f += 1;
            id
        });

        let apikey_item = APIKEY_ITEMS.with_borrow_mut(|items| {
            let apikey_item = ApiKeyItem {
                id,
                title: req_body.title,
                completed: false,
            };

            items.insert(id, apikey_item.clone());
            apikey_item
        });

        let body = CreateApiKeyResponse::ok(&apikey_item).encode();
        create_response(StatusCode::CREATED, body)
    }

    pub fn update_apikey_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let req_body: UpdateApiKeyRequest = json_decode(req.body());
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        APIKEY_ITEMS.with_borrow_mut(|items| {
            let item = items.get_mut(&id).unwrap();

            if let Some(title) = req_body.title {
                item.title = title;
            }

            if let Some(completed) = req_body.completed {
                item.completed = completed;
            }
        });

        let body = UpdateApiKeyResponse::ok(&()).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn delete_apikey_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let req_body: DeleteApiKeyRequest = json_decode(req.body());

        let id = req_body.id;

        APIKEY_ITEMS.with_borrow_mut(|items| {
            items.remove(&id);
        });

        let deleted_data = DeletedApiKeyData {
            deleted_id: req_body.id,
        };

        let body = DeleteApiKeyResponse::ok(&deleted_data).encode();
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