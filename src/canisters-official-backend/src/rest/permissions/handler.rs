// src/rest/permissions/handler.rs


pub mod permissions_handlers {
    use crate::{
        core::{api::{internals::drive_internals::{can_user_access_permission, check_directory_permissions, get_inherited_resources_list, has_manage_permission, parse_directory_grantee_id, parse_directory_resource_id}, uuid::generate_unique_id}, state::{directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, drives::state::state::OWNER_ID, permissions::{state::state::{PERMISSIONS_BY_ID_HASHTABLE, PERMISSIONS_BY_RESOURCE_HASHTABLE}, types::{DirectoryGranteeID, DirectoryPermissionID, DirectoryPermissionType}}, teams::state::state::{is_team_admin, is_user_on_team}}, types::UserID}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::DirectoryResourceID, permissions::types::{CheckPermissionResult, DeletePermissionRequest, DeletePermissionResponseData, ErrorResponse, PermissionCheckRequest}},
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_permissions_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        // 2. Get permission ID from path params
        let permission_id = match params.get("directory_permission_id") {
            Some(id) => DirectoryPermissionID(id.to_string()),
            None => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Missing permission ID".to_string()).encode()
            ),
        };
        // 3. Look up permission in state
        let permission = PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow().get(&permission_id).cloned()
        });

        // 4. Verify access rights using helper function
        match &permission {
            Some(p) => {
                let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
                
                if !can_user_access_permission(&requester_api_key.user_id, p, is_owner) {
                    return create_auth_error_response();
                }
            }
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        }

        // 5. Return permission if found and authorized
        match permission {
            Some(permission) => create_response(
                StatusCode::OK,
                serde_json::to_vec(&permission).expect("Failed to serialize permission")
            ),
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        }

    }

    pub fn check_permissions_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_response(
                StatusCode::UNAUTHORIZED,
                ErrorResponse::unauthorized().encode()
            ),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let check_request = match serde_json::from_slice::<PermissionCheckRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        // Validate resource ID format
        let resource_id = match parse_directory_resource_id(&check_request.resource_id.to_string()) {
            Ok(id) => id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid resource ID format".to_string()).encode()
            ),
        };

        // Validate grantee ID format
        let grantee_id = match parse_directory_grantee_id(&check_request.grantee_id.to_string()) {
            Ok(id) => id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid grantee ID format".to_string()).encode()
            ),
        };
    
        // 3. Check if requester is authorized to check these permissions
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        let is_authorized = if is_owner {
            true
        } else {
            match &grantee_id {
                DirectoryGranteeID::User(user_id) if user_id.0 == requester_api_key.user_id.0 => true,
                DirectoryGranteeID::Team(team_id) => {
                    is_team_admin(&requester_api_key.user_id, team_id) && 
                    is_user_on_team(&UserID(grantee_id.to_string()), team_id)
                },
                _ => has_manage_permission(&requester_api_key.user_id, &resource_id)
            }
        };

        if !is_authorized {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to check permissions for this grantee".to_string()).encode()
            );
        }

        // 4. Check if the resource exists
        let resource_exists = match &resource_id {
            DirectoryResourceID::File(file_id) => {
                file_uuid_to_metadata.contains_key(file_id)
            },
            DirectoryResourceID::Folder(folder_id) => {
                folder_uuid_to_metadata.contains_key(folder_id)
            }
        };

        if !resource_exists {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, format!("Resource {} not found", resource_id)).encode()
            );
        }

        // 5. Check permissions using our helper function
        let permissions = check_directory_permissions(
            resource_id.clone(),
            grantee_id.clone()
        );

        // 6. Create and return the success response
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&CheckPermissionResult {
                resource_id,
                grantee_id,
                permissions,
            }).expect("Failed to serialize response")
        )
    }

    pub fn upsert_permissions_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        
    }

    pub fn delete_permissions_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = req.body();
        let delete_request = match serde_json::from_slice::<DeletePermissionRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // 3. Check if permission exists and get it
        let permission = PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow().get(&delete_request.permission_id).cloned()
        });
    
        let permission = match permission {
            Some(p) => p,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        };
    
        // 4. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        let is_granter = permission.granted_by == requester_api_key.user_id;
        
        // Check manage permissions on the resource and all its parents
        let resources_to_check = get_inherited_resources_list(permission.resource_id.clone());
        let has_manage = resources_to_check.iter().any(|resource_id| {
            check_directory_permissions(
                resource_id.clone(),
                DirectoryGranteeID::User(requester_api_key.user_id.clone())
            ).contains(&DirectoryPermissionType::Manage)
        });
    
        if !is_owner && !is_granter && !has_manage {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to delete this permission".to_string()).encode()
            );
        }
    
        // 5. Delete the permission from all indices
        PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().remove(&delete_request.permission_id);
        });
    
        PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            if let Some(permission_set) = permissions_by_resource.borrow_mut().get_mut(&permission.resource_id) {
                permission_set.remove(&delete_request.permission_id);
            }
        });
    
        // 6. Return success response
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&DeletePermissionResponseData {
                deleted_id: delete_request.permission_id,
            }).expect("Failed to serialize response")
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