// src/rest/directorys/handler.rs


pub mod directorys_handlers {
    use crate::{
        core::{api::{drive::drive::fetch_files_at_folder_path, uuid::generate_unique_id}, state::{directory::{}, drives::state::state::OWNER_ID, raw_storage::{state::{get_file_chunks, store_chunk, store_filename, FILE_META}, types::{ChunkId, FileChunk, CHUNK_SIZE}}}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response, create_raw_upload_error_response}, directory::types::{CompleteUploadRequest, CompleteUploadResponse, DirectoryActionRequest, DirectoryActionResponse, DirectoryListResponse, ErrorResponse, FileMetadataResponse, ListDirectoryRequest, UploadChunkRequest, UploadChunkResponse}}, 
        
    };
    
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    use urlencoding::decode;
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
        debug_log!("Handling upload chunk request");

        let upload_req: UploadChunkRequest = match serde_json::from_slice(req.body()) {
            Ok(req) => req,
            Err(_) => {
                debug_log!("handle_upload_chunk: Failed to deserialize request");
                return create_raw_upload_error_response("Invalid request format")
            }
        };

        debug_log!("handle_upload_chunk: Handling chunk upload");
        debug_log!("  file_id      = {}", upload_req.file_id);
        debug_log!("  chunk_index  = {}", upload_req.chunk_index);
        debug_log!("  total_chunks = {}", upload_req.total_chunks);
        debug_log!("  chunk_size   = {}", upload_req.chunk_data.len());
    
        if upload_req.chunk_data.len() > CHUNK_SIZE {
            return create_raw_upload_error_response("Chunk too large");
        }
    
        let chunk_id = ChunkId(format!("{}-{}", upload_req.file_id, upload_req.chunk_index));
        
        let chunk = FileChunk {
            id: chunk_id.clone(),
            file_id: upload_req.file_id,
            chunk_index: upload_req.chunk_index,
            data: upload_req.chunk_data.clone(),
            size: upload_req.chunk_data.len()
        };
        debug_log!("handle_upload_chunk: Storing chunk {:?}", chunk.id);
    
        store_chunk(chunk);
    
        let response = UploadChunkResponse {
            chunk_id: chunk_id.0,
            bytes_received: upload_req.chunk_data.len()
        };
    
        debug_log!("handle_upload_chunk: Successfully stored chunk");
        create_success_response(&response)
    }
    
    pub fn handle_complete_upload(req: &HttpRequest, _: &Params) -> HttpResponse<'static> {
        let complete_req: CompleteUploadRequest = match serde_json::from_slice(req.body()) {
            Ok(req) => req,
            Err(_) => return create_raw_upload_error_response("Invalid request format")
        };
        debug_log!("handle_complete_upload: Completing upload");
        debug_log!("  file_id = {}", complete_req.file_id);

        store_filename(&complete_req.file_id, &complete_req.filename);
    
        let chunks = get_file_chunks(&complete_req.file_id);
        debug_log!("handle_complete_upload: Found {} chunks", chunks.len());

        let total_size: usize = chunks.iter().map(|c| c.size).sum();
        debug_log!("handle_complete_upload: Total size = {} bytes", total_size);
    
        let response = CompleteUploadResponse {
            file_id: complete_req.file_id,
            size: total_size,
            chunks: chunks.len() as u32,
            filename: complete_req.filename
        };
         debug_log!("handle_complete_upload: Returning final response with size={} chunks={}", response.size, response.chunks);
    
        create_success_response(&response)
    }

    /// Returns the metadata about a file: total size, total chunks, etc.
    pub fn download_file_metadata_handler(req: &HttpRequest, _: &Params) -> HttpResponse<'static> {
        debug_log!("download_file_metadata_handler: Handling file metadata request");

        // // 1. Optionally authenticate, if required
        // let requester_api_key = match authenticate_request(req) {
        //     Some(key) => key,
        //     None => return create_auth_error_response(),
        // };

        // // 2. Check if user is owner, if that's your policy
        // let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        // if !is_owner {
        //     return create_auth_error_response();
        // }

        // 3. Parse query string for file_id
        let raw_query_string = req.get_query().unwrap_or(Some("".to_string()));
        let query_string = raw_query_string.as_deref().unwrap_or("");
        let query_map = crate::rest::helpers::parse_query_string(&query_string);

        let file_id = match query_map.get("file_id") {
            Some(fid) => fid,
            None => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Missing file_id in query".to_string()).encode()
                );
            }
        };
        let file_id = decode(file_id).unwrap_or_else(|_| file_id.into());

        debug_log!("download_file_metadata_handler: file_id={}", file_id);

        // 4. Collect chunks for this file, if any
        let mut chunks = get_file_chunks(&file_id);
        if chunks.is_empty() {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "File not found".to_string()).encode()
            );
        }

        // 5. Sort by chunk index and compute total size
        chunks.sort_by_key(|c| c.chunk_index);
        let total_size: usize = chunks.iter().map(|c| c.size).sum();
        let total_chunks = chunks.len() as u32;

        let filename: String = FILE_META.with(|map| 
            map.borrow()
                .get(&file_id.to_string())
                .clone()  // Change cloned() to clone()
        ).unwrap_or_else(|| "unknown.bin".to_string());

        // Create a JSON response with metadata
        let metadata_response = FileMetadataResponse {
            file_id: file_id.clone().to_string(),
            total_size,
            total_chunks,
            filename
        };

        debug_log!(
            "download_file_metadata_handler: total_size={}, total_chunks={}",
            total_size,
            total_chunks
        );

        create_success_response(&metadata_response)
    }

    /// Returns the data for a single chunk by index.
    pub fn download_file_chunk_handler(req: &HttpRequest, _: &Params) -> HttpResponse<'static> {
        debug_log!("download_file_chunk_handler: Handling file chunk request");

        // // 1. Optionally authenticate
        // let requester_api_key = match authenticate_request(req) {
        //     Some(key) => key,
        //     None => return create_auth_error_response(),
        // };

        // // 2. Owner check, if you want
        // let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        // if !is_owner {
        //     return create_auth_error_response();
        // }

        // 3. Parse query for file_id & chunk_index
        let raw_query_string = req.get_query().unwrap_or(Some("".to_string()));
        let query_string = raw_query_string.as_deref().unwrap_or("");
        let query_map = crate::rest::helpers::parse_query_string(query_string);

        let file_id = match query_map.get("file_id") {
            Some(fid) => fid,
            None => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Missing file_id".to_string()).encode()
                );
            }
        };
        let file_id = decode(file_id).unwrap_or_else(|_| file_id.into());

        let chunk_index_str = match query_map.get("chunk_index") {
            Some(cix) => cix,
            None => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Missing chunk_index".to_string()).encode()
                );
            }
        };
        let chunk_index: u32 = match chunk_index_str.parse() {
            Ok(num) => num,
            Err(_) => {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid chunk_index".to_string()).encode()
                );
            }
        };

        debug_log!("download_file_chunk_handler: file_id={}, chunk_index={}", file_id, chunk_index);

        // 4. Retrieve all chunks, or just the one
        let mut chunks = get_file_chunks(&file_id);
        chunks.sort_by_key(|c| c.chunk_index);

        // Check if chunk_index is valid
        if chunk_index as usize >= chunks.len() {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Chunk index out of range".to_string()).encode()
            );
        }

        let chunk = &chunks[chunk_index as usize];
        debug_log!("download_file_chunk_handler: Found chunk size={}", chunk.size);

        // 5. Return the chunk data in the HTTP body
        //    We'll set the content-type to "application/octet-stream".
        HttpResponse::builder()
            .with_status_code(StatusCode::OK)
            .with_headers(vec![
                ("content-type".to_string(), "application/octet-stream".to_string()),
                ("cache-control".to_string(), "no-store, max-age=0".to_string()),
            ])
            .with_body(chunk.data.clone())
            .build()
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
        let body = serde_json::to_vec(data).expect("Failed to serialize response");
        create_response(StatusCode::OK, body)
    }
    
}