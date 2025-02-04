// src/rest/drives/handler.rs


pub mod drives_handlers {
    use crate::{
        core::{api::uuid::generate_unique_id, state::drives::{types::DriveID}}, debug_log, rest::drives::types::{CreateDriveRequest, CreateDriveResponse, DeleteDriveRequest, DeleteDriveResponse, DeletedDriveData, ErrorResponse, GetDriveResponse, ListDrivesResponse, UpdateDriveRequest, UpdateDriveResponse}
        
    };
    use crate::core::state::drives::{
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_drive_handler(_req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        
    }

    pub fn list_drives_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        
    }

    pub fn create_drive_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        
    }

    pub fn update_drive_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        
    }

    pub fn delete_drive_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        
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