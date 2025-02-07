// src/rest/directorys/handler.rs


pub mod directorys_handlers {
    use crate::{
        core::{api::{drive::drive::fetch_files_at_folder_path, uuid::generate_unique_id}, state::{directory::{}, drives::state::state::OWNER_ID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::{DirectoryActionRequest, DirectoryActionResponse, DirectoryListResponse, ErrorResponse, ListDirectoryRequest}}, 
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn search_directory_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
        }
    
        let response = DirectoryListResponse {
            folders: Vec::new(),
            files: Vec::new(),
            total_folders: 0,
            total_files: 0,
            cursor: None,
        };
    
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&response).expect("Failed to serialize response")
        )
    }

    pub fn list_directorys_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // Only owner can access directories
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
        }
    
        // Parse request body
        let list_request: ListDirectoryRequest = match serde_json::from_slice(request.body()) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        match fetch_files_at_folder_path(list_request) {
            Ok(response) => create_response(
                StatusCode::OK,
                serde_json::to_vec(&response).expect("Failed to serialize response")
            ),
            Err(err) => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, format!("Failed to list directory: {:?}", err)).encode()
            )
        }
    }

    pub fn action_directory_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        if !is_owner {
            return create_auth_error_response();
        }
    
        let action_request: DirectoryActionRequest = match serde_json::from_slice(request.body()) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // Placeholder that returns empty response for all actions
        let response = DirectoryActionResponse {
            data: serde_json::json!({})
        };
    
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&response).expect("Failed to serialize response")
        )
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