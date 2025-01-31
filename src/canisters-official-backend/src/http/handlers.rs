// http/handlers.rs
pub mod todo_handlers {
    use crate::{
        http::certifications::{self, get_certified_response, certify_list_todos_response},
        types::*,
        NEXT_TODO_ID,
        TODO_ITEMS,
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;

    pub fn query_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        certifications::get_certified_response(request)
    }

    pub fn create_todo_item_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let req_body: CreateTodoItemRequest = json_decode(req.body());

        let id = NEXT_TODO_ID.with_borrow_mut(|f| {
            let id = *f;
            *f += 1;
            id
        });

        let todo_item = TODO_ITEMS.with_borrow_mut(|items| {
            let todo_item = TodoItem {
                id,
                title: req_body.title,
                completed: false,
            };

            items.insert(id, todo_item.clone());
            todo_item
        });

        certify_list_todos_response();

        let body = CreateTodoItemResponse::ok(&todo_item).encode();
        create_response(StatusCode::CREATED, body)
    }

    pub fn update_todo_item_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let req_body: UpdateTodoItemRequest = json_decode(req.body());
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        TODO_ITEMS.with_borrow_mut(|items| {
            let item = items.get_mut(&id).unwrap();

            if let Some(title) = req_body.title {
                item.title = title;
            }

            if let Some(completed) = req_body.completed {
                item.completed = completed;
            }
        });

        certify_list_todos_response();

        let body = UpdateTodoItemResponse::ok(&()).encode();
        create_response(StatusCode::OK, body)
    }

    pub fn delete_todo_item_handler(_req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        let id: u32 = params.get("id").unwrap().parse().unwrap();

        TODO_ITEMS.with_borrow_mut(|items| {
            items.remove(&id);
        });

        certify_list_todos_response();

        let body = DeleteTodoItemResponse::ok(&()).encode();
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