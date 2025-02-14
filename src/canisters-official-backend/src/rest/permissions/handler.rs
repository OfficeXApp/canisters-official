// src/rest/permissions/handler.rs


pub mod permissions_handlers {
    use std::collections::HashSet;

    use crate::{
        core::{api::{internals::drive_internals::{can_user_access_permission, check_directory_permissions, get_inherited_resources_list, has_manage_permission, parse_directory_grantee_id, parse_directory_resource_id}, uuid::generate_unique_id}, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::DriveFullFilePath}, drives::state::state::OWNER_ID, permissions::{state::state::{GRANTEE_PERMISSIONS_HASHTABLE, PERMISSIONS_BY_ID_HASHTABLE, PERMISSIONS_BY_RESOURCE_HASHTABLE, PERMISSIONS_BY_TIME_LIST}, types::{PermissionGranteeID, PermissionGranteeType, DirectoryPermission, DirectoryPermissionID, DirectoryPermissionType, PlaceholderPermissionGranteeID}}, teams::state::state::{is_team_admin, is_user_on_team}}, types::{IDPrefix, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::DirectoryResourceID, permissions::types::{CheckPermissionResult, DeletePermissionRequest, DeletePermissionResponseData, ErrorResponse, PermissionCheckRequest, RedeemPermissionRequest, RedeemPermissionResponseData, UpsertPermissionsRequestBody, UpsertPermissionsResponseData}},
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub fn get_directory_permissions_handler(req: &HttpRequest, params: &Params) -> HttpResponse<'static> {
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

    pub fn check_directory_permissions_handler(request: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
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
                PermissionGranteeID::User(user_id) if user_id.0 == requester_api_key.user_id.0 => true,
                PermissionGranteeID::Team(team_id) => {
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

    pub fn upsert_directory_permissions_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(req) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = req.body();
        let upsert_request = match serde_json::from_slice::<UpsertPermissionsRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // 3. Parse and validate resource ID
        let resource_id = match parse_directory_resource_id(&upsert_request.resource_id.to_string()) {
            Ok(id) => id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid resource ID format".to_string()).encode()
            ),
        };
    
        // 4. Parse and validate grantee ID if provided (not required for deferred links)
        let grantee_id = if let Some(grantee) = upsert_request.granted_to {
            match parse_directory_grantee_id(&grantee.to_string()) {
                Ok(id) => id,
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid grantee ID format".to_string()).encode()
                ),
            }
        } else {
            // Create a new deferred link ID for sharing
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(PlaceholderPermissionGranteeID(
                generate_unique_id(IDPrefix::DirectoryShareDeferred, "")
            ))
        };
    
        // 5. Check if resource exists
        let resource_exists = match &resource_id {
            DirectoryResourceID::File(file_id) => file_uuid_to_metadata.contains_key(file_id),
            DirectoryResourceID::Folder(folder_id) => folder_uuid_to_metadata.contains_key(folder_id),
        };
    
        if !resource_exists {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Resource not found".to_string()).encode()
            );
        }
    
        // 6. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id);
        
        let mut allowed_permission_types = if is_owner {
            // Owner can grant any permission
            upsert_request.permission_types.clone()
        } else {
            // Get requester's permissions on the resource and its parents
            let resources_to_check = get_inherited_resources_list(resource_id.clone());
            let requester_permissions: Vec<DirectoryPermissionType> = resources_to_check.iter()
                .flat_map(|resource_id| {
                    check_directory_permissions(
                        resource_id.clone(),
                        PermissionGranteeID::User(requester_api_key.user_id.clone())
                    )
                })
                .collect();
    
            let has_manage = requester_permissions.contains(&DirectoryPermissionType::Manage);
            let has_invite = requester_permissions.contains(&DirectoryPermissionType::Invite);
    
            if !has_manage && !has_invite {
                return create_response(
                    StatusCode::FORBIDDEN,
                    ErrorResponse::err(403, "Not authorized to modify permissions".to_string()).encode()
                );
            }
    
            if has_manage {
                // Can grant any permission if they have manage rights
                upsert_request.permission_types.clone()
            } else {
                // Only include permissions they themselves have
                upsert_request.permission_types.iter()
                    .filter(|&perm| requester_permissions.contains(perm))
                    .cloned()
                    .collect()
            }
        };
    
        let current_time = ic_cdk::api::time() / 1_000_000; // Convert from ns to ms
    
        // 7. Handle update vs create based on ID presence
        if let Some(id) = upsert_request.id {
            // UPDATE case
            let mut existing_permission = match PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| 
                permissions.borrow().get(&id).cloned()
            ) {
                Some(permission) => permission,
                None => return create_response(
                    StatusCode::NOT_FOUND,
                    ErrorResponse::err(404, "Permission not found".to_string()).encode()
                ),
            };
    
            // Update modifiable fields
            existing_permission.permission_types = allowed_permission_types.into_iter().collect();
            existing_permission.begin_date_ms = upsert_request.begin_date_ms.unwrap_or(0);
            existing_permission.expiry_date_ms = upsert_request.expiry_date_ms.unwrap_or(-1);
            existing_permission.inheritable = upsert_request.inheritable;
            existing_permission.note = upsert_request.note.unwrap_or_default();
            existing_permission.last_modified_at = current_time;
    
            // Update state
            PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
                permissions.borrow_mut().insert(id.clone(), existing_permission.clone());
            });
    
            create_response(
                StatusCode::OK,
                serde_json::to_vec(&UpsertPermissionsResponseData {
                    permission: existing_permission,
                }).expect("Failed to serialize response")
            )
        } else {
            // CREATE case
            let permission_id = DirectoryPermissionID(generate_unique_id(IDPrefix::DirectoryPermission, ""));
            
            let new_permission = DirectoryPermission {
                id: permission_id.clone(),
                resource_id: resource_id.clone(),
                resource_path: DriveFullFilePath(resource_id.to_string()),
                grantee_type: match &grantee_id {
                    PermissionGranteeID::Public => PermissionGranteeType::Public,
                    PermissionGranteeID::User(_) => PermissionGranteeType::User,
                    PermissionGranteeID::Team(_) => PermissionGranteeType::Team,
                    PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(_) => PermissionGranteeType::PlaceholderDirectoryPermissionGrantee,
                },
                granted_to: grantee_id.clone(),
                granted_by: requester_api_key.user_id.clone(),
                permission_types: allowed_permission_types.into_iter().collect(),
                begin_date_ms: upsert_request.begin_date_ms.unwrap_or(0),
                expiry_date_ms: upsert_request.expiry_date_ms.unwrap_or(-1),
                inheritable: upsert_request.inheritable,
                note: upsert_request.note.unwrap_or_default(),
                created_at: current_time,
                last_modified_at: current_time,
                from_placeholder_grantee: None,
            };
    
            // Update all state indices
            PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
                permissions.borrow_mut().insert(permission_id.clone(), new_permission.clone());
            });
    
            PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
                permissions_by_resource.borrow_mut()
                    .entry(resource_id)
                    .or_insert_with(HashSet::new)
                    .insert(permission_id.clone());
            });
    
            GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
                grantee_permissions.borrow_mut()
                    .entry(grantee_id)
                    .or_insert_with(HashSet::new)
                    .insert(permission_id.clone());
            });
    
            PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
                permissions_by_time.borrow_mut().push(permission_id);
            });
    
            create_response(
                StatusCode::OK,
                serde_json::to_vec(&UpsertPermissionsResponseData {
                    permission: new_permission,
                }).expect("Failed to serialize response")
            )
        }
    }

    pub fn delete_directory_permissions_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
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
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            ).contains(&DirectoryPermissionType::Manage)
        });
    
        if !is_owner && !is_granter && !has_manage {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to delete this permission".to_string()).encode()
            );
        }
    
        // 5. Delete the permission from all indices
        // Remove from PERMISSIONS_BY_ID_HASHTABLE
        PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().remove(&delete_request.permission_id);
        });

        // Remove from PERMISSIONS_BY_RESOURCE_HASHTABLE
        PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            if let Some(permission_set) = permissions_by_resource.borrow_mut().get_mut(&permission.resource_id) {
                permission_set.remove(&delete_request.permission_id);
                
                // If set is empty, remove the resource entry
                if permission_set.is_empty() {
                    permissions_by_resource.borrow_mut().remove(&permission.resource_id);
                }
            }
        });

        // Remove from GRANTEE_PERMISSIONS_HASHTABLE
        GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            if let Some(permission_set) = grantee_permissions.borrow_mut().get_mut(&permission.granted_to) {
                permission_set.remove(&delete_request.permission_id);
                
                // If set is empty, remove the grantee entry
                if permission_set.is_empty() {
                    grantee_permissions.borrow_mut().remove(&permission.granted_to);
                }
            }
        });

        // Remove from PERMISSIONS_BY_TIME_LIST
        PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
            let mut list = permissions_by_time.borrow_mut();
            if let Some(pos) = list.iter().position(|id| *id == delete_request.permission_id) {
                list.remove(pos);
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

    pub fn redeem_directory_permissions_handler(req: &HttpRequest, _params: &Params) -> HttpResponse<'static> {
        // 1. Parse request body
        let body: &[u8] = req.body();
        let redeem_request = match serde_json::from_slice::<RedeemPermissionRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // 2. Convert permission_id string to DirectoryPermissionID
        let permission_id = DirectoryPermissionID(redeem_request.permission_id);
    
        // 3. Get existing permission
        let mut permission = match PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow().get(&permission_id).cloned()
        }) {
            Some(p) => p,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        };
    
        // 4. Check if permission is actually a one-time link and not already redeemed
        match &permission.granted_to {
            PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(link_id) => {
                if permission.from_placeholder_grantee.is_some() {
                    return create_response(
                        StatusCode::BAD_REQUEST,
                        ErrorResponse::err(400, "Permission has already been redeemed".to_string()).encode()
                    );
                }
                
                // Store the one-time link ID
                permission.from_placeholder_grantee = Some(link_id.clone());
            },
            _ => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Permission is not a one-time link".to_string()).encode()
            ),
        }
    
        // 5. Parse the user_id string into a PermissionGranteeID
        let new_grantee = match parse_directory_grantee_id(&redeem_request.user_id) {
            Ok(grantee_id) => match grantee_id {
                PermissionGranteeID::User(_) => grantee_id,
                _ => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid user ID format".to_string()).encode()
                ),
            },
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid user ID format".to_string()).encode()
            ),
        };
    
        // 6. Update permission and state
        let old_grantee = permission.granted_to.clone();
        permission.granted_to = new_grantee.clone();
        permission.grantee_type = PermissionGranteeType::User;
        permission.last_modified_at = ic_cdk::api::time() / 1_000_000; // Convert ns to ms
    
        // Update all state tables
        PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(permission_id.clone(), permission.clone());
        });
    
        // Update grantee permissions - remove old, add new
        GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut table = grantee_permissions.borrow_mut();
            // Remove from old grantee's set
            table.remove(&old_grantee);
            // Add to new grantee's set
            table.entry(new_grantee)
                .or_insert_with(HashSet::new)
                .insert(permission_id);
        });
    
        // 7. Return updated permission
        create_response(
            StatusCode::OK,
            serde_json::to_vec(&RedeemPermissionResponseData {
                permission,
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