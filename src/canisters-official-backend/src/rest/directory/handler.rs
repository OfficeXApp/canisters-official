// src/rest/directorys/handler.rs


pub mod directorys_handlers {
    use crate::{
        core::{api::{drive::drive::fetch_files_at_folder_path, uuid::generate_unique_id}, state::{directory::{}, drives::state::state::OWNER_ID, raw_storage::{state::{get_file_chunks, get_filename, store_chunk, store_filename, FILE_CHUNKS, FILE_META}, types::{ChunkId, FileChunk, CHUNK_SIZE}}}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response, create_raw_upload_error_response}, directory::types::{CompleteUploadRequest, CompleteUploadResponse, DirectoryActionRequest, DirectoryActionResponse, DirectoryListResponse, ErrorResponse, FileMetadataResponse, ListDirectoryRequest, UploadChunkRequest, UploadChunkResponse}}, 
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    use urlencoding::decode;
    use url::Url;
    use hyperx::header::{ContentRangeSpec, Range};
    use std::str::FromStr;

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

    pub fn handle_upload_chunk(req: &HttpRequest, _: &Params) -> HttpResponse<'static> {
        let upload_req: UploadChunkRequest = match serde_json::from_slice(req.body()) {
            Ok(req) => req,
            Err(_) => return create_error_response("Invalid request format")
        };

        let chunk_id = ChunkId(format!("{}-{}", upload_req.file_id, upload_req.chunk_index));
        
        let chunk = FileChunk {
            id: chunk_id.clone(),
            file_id: upload_req.file_id,
            chunk_index: upload_req.chunk_index,
            data: upload_req.chunk_data.clone(),
            size: upload_req.chunk_data.len()
        };

        store_chunk(chunk);

        let response = UploadChunkResponse {
            chunk_id: chunk_id.0,
            bytes_received: upload_req.chunk_data.len()
        };

        create_success_response(&response)
    }

    pub fn handle_complete_upload(req: &HttpRequest, _: &Params) -> HttpResponse<'static> {
        let complete_req: CompleteUploadRequest = match serde_json::from_slice(req.body()) {
            Ok(req) => req,
            Err(_) => return create_error_response("Invalid request format")
        };

        store_filename(&complete_req.file_id, &complete_req.filename);

        let chunks = get_file_chunks(&complete_req.file_id);
        let total_size: usize = chunks.iter().map(|c| c.size).sum();

        let response = CompleteUploadResponse {
            file_id: complete_req.file_id,
            size: total_size,
            chunks: chunks.len() as u32,
            filename: complete_req.filename
        };

        create_success_response(&response)
    }

    pub fn serve_asset(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        debug_log!("Serving asset from path: {}", req.url());
        
        // Extract file ID
        let file_id = match params.get("file_id") {
            Some(id) => id,
            None => return create_error_response("Missing file ID")
        };
        debug_log!("Serving asset with ID: {}", file_id);
    
        // Get chunks info
        let chunks = get_file_chunks(file_id);
        if chunks.is_empty() {
            return create_error_response("File not found");
        }
    
        // Sort chunks by index
        let mut chunks = chunks;
        chunks.sort_by_key(|c| c.chunk_index);
        
        // Calculate total size
        let total_size: usize = chunks.iter().map(|c| c.size).sum();
        debug_log!("Total file size: {}", total_size);
    
        // Get filename and content type
        let filename = get_filename(file_id).unwrap_or_else(|| "download".to_string());
        let content_type = get_content_type(&filename);
    
        // Create base headers
        let mut headers = vec![
            ("accept-ranges".to_string(), "bytes".to_string()),
            ("content-type".to_string(), content_type),
            ("access-control-allow-origin".to_string(), "*".to_string()),
            ("cache-control".to_string(), "public, max-age=31536000".to_string()),
        ];
    
        // Add content-disposition only for download requests (not media)
        if req.headers().iter().any(|(k, v)| 
            k.to_lowercase() == "sec-fetch-dest" && v.to_lowercase() == "document") {
            headers.push(("content-disposition".to_string(), 
                format!("attachment; filename=\"{}\"", filename)));
        }
    
        // Handle HEAD requests
        if req.method() == "HEAD" {
            headers.push(("content-length".to_string(), total_size.to_string()));
            return HttpResponse::builder()
                .with_status_code(StatusCode::OK)
                .with_headers(headers)
                .with_body(Vec::new())
                .build();
        }
    
        // Check for range request
        if let Some(range_header) = req.headers().iter().find(|(k, _)| k.to_lowercase() == "range") {
            match Range::from_str(&range_header.1) {
                Ok(Range::Bytes(ranges)) if ranges.len() == 1 => {
                    let range = ranges[0].to_satisfiable_range(total_size as u64)
                        .ok_or("Range out of bounds");
    
                    match range {
                        Ok((start, end)) => {
                            // Ensure the range size is within limits
                            let range_size = end - start + 1;
                            if range_size > 2_000_000 {
                                // Adjust end to stay within 2MB limit
                                let end = start + 1_999_999;
                                headers.push(("content-range".to_string(), 
                                    format!("bytes {}-{}/{}", start, end, total_size)));
                                
                                let body = get_range_data(&chunks, start as usize, end as usize);
                                headers.push(("content-length".to_string(), body.len().to_string()));
    
                                return HttpResponse::builder()
                                    .with_status_code(StatusCode::PARTIAL_CONTENT)
                                    .with_headers(headers)
                                    .with_body(body)
                                    .build();
                            }
    
                            let body = get_range_data(&chunks, start as usize, end as usize);
                            headers.push(("content-range".to_string(), 
                                format!("bytes {}-{}/{}", start, end, total_size)));
                            headers.push(("content-length".to_string(), body.len().to_string()));
    
                            return HttpResponse::builder()
                                .with_status_code(StatusCode::PARTIAL_CONTENT)
                                .with_headers(headers)
                                .with_body(body)
                                .build();
                        },
                        Err(_) => {
                            return create_error_response("Invalid range");
                        }
                    }
                },
                _ => return create_error_response("Invalid range request")
            }
        }
    
        // For small files, return everything
        if total_size <= 1_999_999 {
            let mut body = Vec::with_capacity(total_size);
            for chunk in chunks {
                body.extend_from_slice(&chunk.data);
            }
    
            headers.push(("content-length".to_string(), body.len().to_string()));
    
            return HttpResponse::builder()
                .with_status_code(StatusCode::OK)
                .with_headers(headers)
                .with_body(body)
                .build();
        }
    
        // For large files without range, return first 2MB and signal more available
        let first_range_end = 1_999_999;
        let body = get_range_data(&chunks, 0, first_range_end);
        
        headers.push(("content-range".to_string(), 
            format!("bytes 0-{}/{}", first_range_end, total_size)));
        headers.push(("content-length".to_string(), body.len().to_string()));
    
        HttpResponse::builder()
            .with_status_code(StatusCode::PARTIAL_CONTENT)
            .with_headers(headers)
            .with_body(body)
            .build()
    }
    
    // Helper function to get data for a specific byte range
    fn get_range_data(chunks: &[FileChunk], start: usize, end: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(end - start + 1);
        let start_chunk = start / CHUNK_SIZE;
        let end_chunk = end / CHUNK_SIZE;
        let start_offset = start % CHUNK_SIZE;
        let end_offset = end % CHUNK_SIZE;
    
        for chunk_idx in start_chunk..=end_chunk {
            if let Some(chunk) = chunks.iter().find(|c| c.chunk_index == chunk_idx as u32) {
                let chunk_start = if chunk_idx == start_chunk { start_offset } else { 0 };
                let chunk_end = if chunk_idx == end_chunk {
                    std::cmp::min(end_offset + 1, chunk.data.len())
                } else {
                    chunk.data.len()
                };
    
                if chunk_start < chunk.data.len() {
                    result.extend_from_slice(&chunk.data[chunk_start..chunk_end]);
                }
            }
        }
        
        result
    }
    
    fn get_content_type(filename: &str) -> String {
        let extension = filename.split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "mp4" => "video/mp4",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "xls" | "xlsx" => "application/vnd.ms-excel",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }.to_string()
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


    fn create_success_response<T: serde::Serialize>(data: &T) -> HttpResponse<'static> {
        HttpResponse::builder()
            .with_status_code(StatusCode::OK)
            .with_headers(vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("cache-control".to_string(), "no-store, max-age=0".to_string()),
            ])
            .with_body(serde_json::to_vec(data).unwrap())
            .build()
    }

    fn create_error_response(msg: &str) -> HttpResponse<'static> {
        HttpResponse::builder()
            .with_status_code(StatusCode::BAD_REQUEST)
            .with_headers(vec![
                ("content-type".to_string(), "application/json".to_string()),
            ])
            .with_body(format!("{{\"error\":\"{}\"}}", msg).into_bytes())
            .build()
    }
    
}