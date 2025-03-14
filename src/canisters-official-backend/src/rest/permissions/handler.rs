// src/rest/permissions/handler.rs


pub mod permissions_handlers {
    use std::collections::HashSet;

    use crate::{
        core::{api::{permissions::{directory::{can_user_access_directory_permission, check_directory_permissions, get_inherited_resources_list, has_directory_manage_permission, parse_directory_resource_id, parse_permission_grantee_id}, system::{can_user_access_system_permission, check_permissions_table_access, check_system_permissions, has_system_manage_permission}}, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}}, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::DriveFullFilePath}, drives::{state::state::{update_external_id_mapping, OWNER_ID}, types::{ExternalID, ExternalPayload}}, groups::state::state::{is_group_admin, is_user_on_group}, permissions::{state::state::{DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE, DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE, DIRECTORY_PERMISSIONS_BY_TIME_LIST, SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE, SYSTEM_PERMISSIONS_BY_TIME_LIST}, types::{DirectoryPermission, DirectoryPermissionID, DirectoryPermissionType, PermissionGranteeID, PlaceholderPermissionGranteeID, SystemPermission, SystemPermissionID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, tags::types::redact_tag}, types::{IDPrefix, UserID}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, directory::types::DirectoryResourceID, permissions::types::{CheckPermissionResponse, CheckPermissionResult, CheckSystemPermissionResponse, CheckSystemPermissionResult, CreateDirectoryPermissionsRequestBody, CreateDirectoryPermissionsResponseData, CreatePermissionsResponse, CreateSystemPermissionsRequestBody, CreateSystemPermissionsResponse, CreateSystemPermissionsResponseData, DeletePermissionRequest, DeletePermissionResponse, DeletePermissionResponseData, DeleteSystemPermissionRequest, DeleteSystemPermissionResponse, DeleteSystemPermissionResponseData, ErrorResponse, GetPermissionResponse, GetSystemPermissionResponse, ListSystemPermissionsRequestBody, ListSystemPermissionsRequestBodyFilters, ListSystemPermissionsResponse, ListSystemPermissionsResponseData, PermissionCheckRequest, RedeemPermissionRequest, RedeemPermissionResponse, RedeemPermissionResponseData, RedeemSystemPermissionRequest, RedeemSystemPermissionResponse, RedeemSystemPermissionResponseData, SystemPermissionCheckRequest, UpdateDirectoryPermissionsRequestBody, UpdateDirectoryPermissionsResponseData, UpdatePermissionsResponse, UpdateSystemPermissionsRequestBody, UpdateSystemPermissionsResponse, UpdateSystemPermissionsResponseData}, webhooks::types::SortDirection},
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_directory_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
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
        let permission = DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow().get(&permission_id).cloned()
        });

        // 4. Verify access rights using helper function
        match &permission {
            Some(p) => {
                let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
                
                if !can_user_access_directory_permission(&requester_api_key.user_id, p, is_owner) {
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
                GetPermissionResponse::ok(&permission.cast_fe(&requester_api_key.user_id)).encode()
            ),
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        }

    }

    pub async fn check_directory_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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
        if let Err(e) = check_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }

        // Validate resource ID format
        let resource_id = match parse_directory_resource_id(&check_request.resource_id.to_string()) {
            Ok(id) => id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid resource ID format".to_string()).encode()
            ),
        };

        // Validate grantee ID format
        let grantee_id = match parse_permission_grantee_id(&check_request.grantee_id.to_string()) {
            Ok(id) => id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid grantee ID format".to_string()).encode()
            ),
        };
    
        // 3. Check if requester is authorized to check these permissions
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        let is_authorized = if is_owner {
            true
        } else {
            match &grantee_id {
                PermissionGranteeID::User(user_id) if user_id.0 == requester_api_key.user_id.0 => true,
                PermissionGranteeID::Group(group_id) => {
                    is_group_admin(&requester_api_key.user_id, group_id) && 
                    is_user_on_group(&UserID(grantee_id.to_string()), group_id).await
                },
                _ => has_directory_manage_permission(&requester_api_key.user_id, &resource_id).await
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
        ).await;



        create_response(
            StatusCode::OK,
            CheckPermissionResponse::ok(&CheckPermissionResult {
                resource_id: resource_id.to_string(),
                grantee_id: grantee_id.to_string(),
                permissions,
            }).encode()
        )
    }

    pub async fn create_directory_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let upsert_request = match serde_json::from_slice::<CreateDirectoryPermissionsRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(e) = upsert_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
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
            match parse_permission_grantee_id(&grantee.to_string()) {
                Ok(id) => id,
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid grantee ID format".to_string()).encode()
                ),
            }
        } else {
            // Create a new deferred link ID for sharing
            let _placeholder_id = PlaceholderPermissionGranteeID(
                generate_uuidv4(IDPrefix::PlaceholderPermissionGrantee)
            );
            let _placeholder_grantee = PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(_placeholder_id.clone());
            mark_claimed_uuid(&_placeholder_id.clone().to_string());
            _placeholder_grantee
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        let mut allowed_permission_types = if is_owner {
            // Owner can grant any permission
            upsert_request.permission_types.clone()
        } else {
            // Get requester's permissions on the resource and its parents
            let resources_to_check = get_inherited_resources_list(resource_id.clone());
            let mut requester_permissions = Vec::new();
            for resource_id in resources_to_check.iter() {
                let permissions = check_directory_permissions(
                    resource_id.clone(),
                    PermissionGranteeID::User(requester_api_key.user_id.clone())
                ).await;
                requester_permissions.extend(permissions);
            }
    
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

        let prestate = snapshot_prestate();
    
        // 7. Handle update vs create based on ID presence
        
        // CREATE case
        let permission_id = match upsert_request.id {
            Some(id) => DirectoryPermissionID(id.to_string()),
            None => DirectoryPermissionID(generate_uuidv4(IDPrefix::DirectoryPermission)),
        };
        
        let new_permission = DirectoryPermission {
            id: permission_id.clone(),
            resource_id: resource_id.clone(),
            resource_path: DriveFullFilePath(resource_id.to_string()),
            granted_to: grantee_id.clone(),
            granted_by: requester_api_key.user_id.clone(),
            permission_types: allowed_permission_types.into_iter().collect(),
            begin_date_ms: upsert_request.begin_date_ms.unwrap_or(0),
            expiry_date_ms: upsert_request.expiry_date_ms.unwrap_or(-1),
            inheritable: upsert_request.inheritable.unwrap_or(true),
            note: upsert_request.note.unwrap_or_default(),
            created_at: current_time,
            last_modified_at: current_time,
            from_placeholder_grantee: None,
            tags: vec![],
            external_id: Some(ExternalID(upsert_request.external_id.clone().unwrap_or_default())),
            external_payload: Some(ExternalPayload(upsert_request.external_payload.clone().unwrap_or_default())),
        };

        // Update all state indices
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(permission_id.clone(), new_permission.clone());
        });

        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            permissions_by_resource.borrow_mut()
                .entry(resource_id)
                .or_insert_with(Vec::new)
                .push(permission_id.clone());
        });

        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            grantee_permissions.borrow_mut()
                .entry(grantee_id)
                .or_insert_with(Vec::new)
                .push(permission_id.clone());
        });

        DIRECTORY_PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
            permissions_by_time.borrow_mut().push(permission_id.clone());
        });

        mark_claimed_uuid(&permission_id.clone().to_string());

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Directory Permission {}", 
                requester_api_key.user_id,
                permission_id.clone()
            ).to_string()
        ));




        create_response(
            StatusCode::OK,
            CreatePermissionsResponse::ok(&CreateDirectoryPermissionsResponseData {
                permission: new_permission.cast_fe(&requester_api_key.user_id.clone()),
            }).encode()
        )
        
    }

    pub async fn update_directory_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let upsert_request = match serde_json::from_slice::<UpdateDirectoryPermissionsRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(e) = upsert_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    

        // 7. Handle update vs create based on ID presence
        // UPDATE case
        let id = upsert_request.id;
        let mut existing_permission = match DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| 
            permissions.borrow().get(&id).cloned()
        ) {
            Some(permission) => permission,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        };

    
        // 6. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        let mut allowed_permission_types = if is_owner {
            // Owner can grant any permission
            upsert_request.permission_types.clone()
        } else {
            // Get requester's permissions on the resource and its parents
            let resources_to_check = get_inherited_resources_list(existing_permission.resource_id.clone());
            let mut requester_permissions = Vec::new();
            for resource_id in resources_to_check.iter() {
                let permissions = check_directory_permissions(
                    resource_id.clone(),
                    PermissionGranteeID::User(requester_api_key.user_id.clone())
                ).await;
                requester_permissions.extend(permissions);
            }
    
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

        let prestate = snapshot_prestate();
    
        // Update modifiable fields
        existing_permission.permission_types = allowed_permission_types
                                                    .into_iter()
                                                    .collect::<HashSet<_>>()
                                                    .into_iter()
                                                    .collect();
        existing_permission.begin_date_ms = upsert_request.begin_date_ms.unwrap_or(0);
        existing_permission.expiry_date_ms = upsert_request.expiry_date_ms.unwrap_or(-1);
        existing_permission.inheritable = upsert_request.inheritable.unwrap_or(true);
        existing_permission.note = upsert_request.note.unwrap_or_default();
        existing_permission.last_modified_at = current_time;

        // Update state
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(id.clone(), existing_permission.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Directory Permission {}", 
                requester_api_key.user_id,
                id.0
            ).to_string()
        ));


        create_response(
            StatusCode::OK,
            UpdatePermissionsResponse::ok(&UpdateDirectoryPermissionsResponseData {
                permission: existing_permission.cast_fe(&requester_api_key.user_id.clone()),
            }).encode()
        )
        
    }

    pub async fn delete_directory_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeletePermissionRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(e) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
        // 3. Check if permission exists and get it
        let permission = DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
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
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        let is_granter = permission.granted_by == requester_api_key.user_id;
        
        // Check manage permissions on the resource and all its parents
        let resources_to_check = get_inherited_resources_list(permission.resource_id.clone());
        let mut has_manage = false;
        for resource_id in resources_to_check.iter() {
            let permissions = check_directory_permissions(
                resource_id.clone(),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            ).await;
            
            if permissions.contains(&DirectoryPermissionType::Manage) {
                has_manage = true;
                break;
            }
        }
    
        if !is_owner && !is_granter && !has_manage {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to delete this permission".to_string()).encode()
            );
        }

        let prestate = snapshot_prestate();
    
        // 5. Delete the permission from all indices
        // Remove from DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().remove(&delete_request.permission_id);
        });

        // Remove from DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE
        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            if let Some(permission_vec) = permissions_by_resource.borrow_mut().get_mut(&permission.resource_id) {
                *permission_vec = permission_vec.iter().filter(|id| **id != delete_request.permission_id).cloned().collect();
                // If set is empty, remove the resource entry
                if permission_vec.is_empty() {
                    permissions_by_resource.borrow_mut().remove(&permission.resource_id);
                }
            }
        });

        // Remove from DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE
        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            if let Some(permission_vec) = grantee_permissions.borrow_mut().get_mut(&permission.granted_to) {
                *permission_vec = permission_vec.iter().filter(|id| **id != delete_request.permission_id).cloned().collect();
                // If set is empty, remove the grantee entry
                if permission_vec.is_empty() {
                    grantee_permissions.borrow_mut().remove(&permission.granted_to);
                }
            }
        });

        // Remove from DIRECTORY_PERMISSIONS_BY_TIME_LIST
        DIRECTORY_PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
            let mut list = permissions_by_time.borrow_mut();
            if let Some(pos) = list.iter().position(|id| *id == delete_request.permission_id) {
                list.remove(pos);
            }
        });
    
        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Directory Permission {}", 
                requester_api_key.user_id,
                delete_request.permission_id.0
            ).to_string()
        ));



        create_response(
            StatusCode::OK,
            DeletePermissionResponse::ok(&DeletePermissionResponseData {
                deleted_id: delete_request.permission_id,
            }).encode()
        )
    }

    pub async fn redeem_directory_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        
        // 1. Parse request body
        let body: &[u8] = request.body();
        let redeem_request = match serde_json::from_slice::<RedeemPermissionRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
        if let Err(e) = redeem_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
     
        // 2. Convert permission_id string to DirectoryPermissionID
        let permission_id = DirectoryPermissionID(redeem_request.permission_id);
    
        // 3. Get existing permission
        let mut permission = match DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
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
        let new_grantee = match parse_permission_grantee_id(&redeem_request.user_id) {
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

        let prestate = snapshot_prestate();
    
        // 6. Update permission and state
        let old_grantee = permission.granted_to.clone();
        permission.granted_to = new_grantee.clone();
        permission.last_modified_at = ic_cdk::api::time() / 1_000_000; // Convert ns to ms
    
        // Update all state tables
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(permission_id.clone(), permission.clone());
        });
    
        // Update grantee permissions - remove old, add new
        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut table = grantee_permissions.borrow_mut();
            // Remove from old grantee's set
            table.remove(&old_grantee);
            // Add to new grantee's set 
            table.entry(new_grantee)
                .or_insert_with(Vec::new)
                .push(permission_id.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Redeem Directory Permission {}", 
                requester_api_key.user_id,
                permission_id.0
            ).to_string()
        ));
    

        create_response(
            StatusCode::OK,
            RedeemPermissionResponse::ok(
                &RedeemPermissionResponseData {
                    permission: permission.cast_fe(&requester_api_key.user_id.clone())
                }
            ).encode()
        )
    }

    pub async fn get_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Get permission ID from path params
        let permission_id = match params.get("system_permission_id") {
            Some(id) => SystemPermissionID(id.to_string()),
            None => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Missing permission ID".to_string()).encode()
            ),
        };
    
        // 3. Look up permission in state
        let permission = SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow().get(&permission_id).cloned()
        });

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        // 4. First check table-level permission
        if !check_permissions_table_access(&requester_api_key.user_id, SystemPermissionType::View, is_owner) {
            return create_auth_error_response();
        }
    
        // 4. Verify access rights
        match &permission {
            Some(p) => {
                let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
                
                if !can_user_access_system_permission(&requester_api_key.user_id, p, is_owner) {
                    return create_auth_error_response();
                }
            }
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        }
        

        match permission {
            Some(permission) => create_response(
                StatusCode::OK,
                GetSystemPermissionResponse::ok(&permission.cast_fe(&requester_api_key.user_id)).encode()
            ),
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        }
    }
  

    pub async fn list_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let request_body = match serde_json::from_slice::<ListSystemPermissionsRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        // 3. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
        // Check table-level permissions if not owner
        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Permissions);
            let permissions = check_system_permissions(
                resource_id.clone(),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            
            if !permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }
    
        // 4. Parse cursor if provided
        let cursor = if let Some(cursor_str) = &request_body.cursor {
            match cursor_str.parse::<usize>() {
                Ok(idx) => Some(idx),
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid cursor format".to_string()).encode()
                ),
            }
        } else {
            None
        };
    
        // 5. Collect matching permissions with pagination applied directly
        let user_id = &requester_api_key.user_id;
        let mut filtered_permissions = Vec::new();
        let page_size = request_body.page_size;
        let direction = request_body.direction;
        
        // Use different strategies based on filters
        match &request_body.filters.resource_ids {
            Some(resource_ids) if !resource_ids.is_empty() => {
                // Process resource IDs directly
                let mut total_processed = 0;
                
                for resource_id in resource_ids {
                    // Skip if user doesn't have permission to view this resource (unless owner)
                    if !is_owner && !has_system_manage_permission(user_id, resource_id) {
                        continue;
                    }
                    
                    // Get permissions for this resource
                    SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
                        if let Some(permission_ids) = permissions_by_resource.borrow().get(resource_id) {
                            // Clone to avoid borrow issues in nested closures
                            let permission_ids = permission_ids.clone();
                            
                            // Sort permission IDs by time
                            let mut timed_ids: Vec<(u64, SystemPermissionID)> = Vec::new();
                            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|id_store| {
                                let id_store = id_store.borrow();
                                for id in &permission_ids {
                                    if let Some(permission) = id_store.get(id) {
                                        timed_ids.push((permission.created_at, id.clone()));
                                    }
                                }
                            });
                            
                            // Sort based on direction
                            match direction {
                                SortDirection::Desc => timed_ids.sort_by(|a, b| b.0.cmp(&a.0)), // Newest first
                                SortDirection::Asc => timed_ids.sort_by(|a, b| a.0.cmp(&b.0)),  // Oldest first
                            }
                            
                            // Skip items before cursor if needed
                            let start_idx = cursor.unwrap_or(0);
                            
                            // Skip items we've already processed
                            let adjusted_start = if start_idx > total_processed {
                                start_idx - total_processed
                            } else {
                                0
                            };
                            
                            // Only process items within our pagination window
                            let items_to_process = &timed_ids[adjusted_start.min(timed_ids.len())..];
                            total_processed += timed_ids.len();
                            
                            for (_, permission_id) in items_to_process {
                                SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|id_store| {
                                    if let Some(permission) = id_store.borrow().get(permission_id) {
                                        // Apply remaining filters
                                        if passes_remaining_filters(permission, &request_body.filters, user_id, is_owner) {
                                            filtered_permissions.push(permission.clone());
                                        }
                                    }
                                });
                                
                                // Early exit if we have enough items
                                if filtered_permissions.len() >= page_size {
                                    break;
                                }
                            }
                        }
                    });
                    
                    // Early exit if we have enough items
                    if filtered_permissions.len() >= page_size {
                        break;
                    }
                }
            },
            _ => {
                // Process all permissions in time order
                SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|time_list| {
                    let time_list = time_list.borrow();
                    let total_permissions = time_list.len();
                    
                    // Skip processing if no permissions
                    if total_permissions == 0 {
                        return;
                    }
                    
                    // Determine start index and direction
                    let (start_idx, step): (usize, isize) = match direction {
                        SortDirection::Desc => {
                            // For desc, we start from newest (end of list) and go backwards
                            let start = if let Some(c) = cursor {
                                (total_permissions as isize - 1 - c as isize).max(0) as usize
                            } else {
                                total_permissions - 1 // Start from newest
                            };
                            (start, -1)
                        },
                        SortDirection::Asc => {
                            // For asc, we start from oldest (start of list) and go forwards
                            let start = if let Some(c) = cursor {
                                c.min(total_permissions - 1)
                            } else {
                                0 // Start from oldest
                            };
                            (start, 1)
                        }
                    };
                    
                    // Process permissions with early exit conditions
                    SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|id_store| {
                        let id_store = id_store.borrow();
                        let mut idx = start_idx;
                        
                        for _ in 0..page_size {
                            if let Some(permission) = id_store.get(&time_list[idx]) {
                                // Check if user has access to the resource
                                if is_owner || has_system_manage_permission(user_id, &permission.resource_id) {
                                    // Apply remaining filters
                                    if passes_remaining_filters(permission, &request_body.filters, user_id, is_owner) {
                                        filtered_permissions.push(permission.clone());
                                    }
                                }
                            }
                            
                            // Move to next item based on direction
                            let next_idx = (idx as isize + step) as usize;
                            if next_idx >= total_permissions {
                                break; // Reached the end
                            }
                            idx = next_idx;
                        }
                    });
                });
            }
        }
        
        // 6. Calculate next cursor for pagination
        let next_cursor = if filtered_permissions.len() >= page_size {
            // There might be more items
            Some((cursor.unwrap_or(0) + page_size).to_string())
        } else {
            None
        };
        
        // 7. Create response with filtered, paginated permissions
        let response_data = ListSystemPermissionsResponseData {
            items: filtered_permissions
                .clone().into_iter()
                .map(|permission| permission.cast_fe(user_id))
                .collect(),
            page_size: filtered_permissions.len(),
            total: SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|h| h.borrow().len()), // Return total count of all permissions
            cursor: next_cursor,
        };
    
        create_response(
            StatusCode::OK,
            ListSystemPermissionsResponse::ok(&response_data).encode()
        )
    }
    
    // Helper function to check remaining filters (after resource_id access check)
    fn passes_remaining_filters(
        permission: &SystemPermission,
        filters: &ListSystemPermissionsRequestBodyFilters,
        requester_id: &UserID,
        is_owner: bool
    ) -> bool {
        // 1. Check if current user has access to this permission
        if !can_user_access_system_permission(requester_id, permission, is_owner) {
            return false;
        }
    
        // 2. Filter by grantee_id if specified
        if let Some(grantee_ids) = &filters.grantee_ids {
            if !grantee_ids.is_empty() && !grantee_ids.contains(&permission.granted_to) {
                return false;
            }
        }
    
        // 3. Filter by tags if specified (OR relationship between tags)
        if let Some(tags) = &filters.tags {
            if !tags.is_empty() {
                // If any tag in the filter matches any tag in the permission, it passes
                let has_matching_tag = tags.iter().any(|filter_tag| {
                    permission.tags.iter().any(|permission_tag| {
                        // Check if user has access to view this tag
                        match redact_tag(permission_tag.clone(), requester_id.clone()) {
                            Some(tag) => &tag == filter_tag,
                            None => false // User cannot see this tag, so it's not a match
                        }
                    })
                });
                
                if !has_matching_tag {
                    return false;
                }
            }
        }
    
        true
    }

    pub async fn create_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let upsert_request = match serde_json::from_slice::<CreateSystemPermissionsRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(e) = upsert_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
        // 3. Parse resource ID string into SystemResourceID
        debug_log!("Upsert request resource_id {:?}", upsert_request.resource_id.clone());
        let resource_id = match upsert_request.resource_id.split_once('_') {
            Some(("TABLE", table_name)) => {
                match table_name {
                    "DRIVES" => SystemResourceID::Table(SystemTableEnum::Drives),
                    "DISKS" => SystemResourceID::Table(SystemTableEnum::Disks),
                    "CONTACTS" => SystemResourceID::Table(SystemTableEnum::Contacts),
                    "GROUPS" => SystemResourceID::Table(SystemTableEnum::Groups),
                    "API_KEYS" => SystemResourceID::Table(SystemTableEnum::Api_Keys),
                    "PERMISSIONS" => SystemResourceID::Table(SystemTableEnum::Permissions),
                    "WEBHOOKS" => SystemResourceID::Table(SystemTableEnum::Webhooks),
                    "TAGS" => SystemResourceID::Table(SystemTableEnum::Tags),
                    _ => return create_response(
                        StatusCode::BAD_REQUEST,
                        ErrorResponse::err(400, "Invalid table name".to_string()).encode()
                    ),
                }
            },
            Some(_) => SystemResourceID::Record(SystemRecordIDEnum::Unknown(upsert_request.resource_id.clone())),
            None => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid resource ID format".to_string()).encode()
            ),
        };
        debug_log!("Prased Upsert request resource_id {:?}", resource_id.clone());
    
        // 4. Parse and validate grantee ID if provided (not required for deferred links)
        let grantee_id = if let Some(grantee) = upsert_request.granted_to {
            match parse_permission_grantee_id(&grantee) {
                Ok(id) => id,
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid grantee ID format".to_string()).encode()
                ),
            }
        } else {
            // Create a new deferred link ID for sharing
            let _placeholder_id = PlaceholderPermissionGranteeID(
                generate_uuidv4(IDPrefix::PlaceholderPermissionGrantee)
            );
            let _placeholder_grantee = PermissionGranteeID::PlaceholderDirectoryPermissionGrantee(_placeholder_id.clone());
            mark_claimed_uuid(&_placeholder_id.clone().to_string());
            _placeholder_grantee
        };
    
        // 5. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
    
        let current_time = ic_cdk::api::time() / 1_000_000; // Convert from ns to ms
    
        
        // CREATE case
        let has_table_permission = check_permissions_table_access(&requester_api_key.user_id, SystemPermissionType::Create, is_owner);
        if !is_owner && !has_system_manage_permission(&requester_api_key.user_id, &resource_id) &&!has_table_permission {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to modify system permissions".to_string()).encode()
            );
        }

        let prestate = snapshot_prestate();


        let permission_id = match upsert_request.id {
            Some(id) => SystemPermissionID(id.to_string()),
            None => SystemPermissionID(generate_uuidv4(IDPrefix::SystemPermission)),
        };
        
        let new_permission = SystemPermission {
            id: permission_id.clone(),
            resource_id: resource_id.clone(),
            granted_to: grantee_id.clone(),
            granted_by: requester_api_key.user_id.clone(),
            permission_types: upsert_request.permission_types.into_iter().collect(),
            begin_date_ms: upsert_request.begin_date_ms.unwrap_or(0),
            expiry_date_ms: upsert_request.expiry_date_ms.unwrap_or(-1),
            note: upsert_request.note.unwrap_or_default(),
            created_at: current_time,
            last_modified_at: current_time,
            from_placeholder_grantee: None,
            tags: vec![],
            metadata: upsert_request.metadata,
            external_id: match upsert_request.external_id {
                Some(id) => Some(ExternalID(id)),
                None => None,
            },
            external_payload: match upsert_request.external_payload {
                Some(payload) => Some(ExternalPayload(payload)),
                None => None,
            },
        };

        // Update all state indices
        SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(permission_id.clone(), new_permission.clone());
        });

        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            permissions_by_resource.borrow_mut()
                .entry(resource_id)
                .or_insert_with(Vec::new)
                .push(permission_id.clone());
        });

        SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            grantee_permissions.borrow_mut()
                .entry(grantee_id)
                .or_insert_with(Vec::new)
                .push(permission_id.clone());
        });

        SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
            permissions_by_time.borrow_mut().push(permission_id.clone());
        });

        mark_claimed_uuid(&permission_id.clone().to_string());

        update_external_id_mapping(
            None,
            new_permission.external_id.clone(),
            Some(new_permission.id.clone().to_string()),
        );

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create System Permission {}", 
                requester_api_key.user_id,
                permission_id.clone()
            ).to_string()
        ));

        let final_permission = CreateSystemPermissionsResponseData {
            permission: new_permission.cast_fe(&requester_api_key.user_id.clone())
        };

        create_response(
            StatusCode::OK,
            CreateSystemPermissionsResponse::ok(&final_permission).encode()
        )
        
    }

    pub async fn update_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let upsert_request = match serde_json::from_slice::<UpdateSystemPermissionsRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(e) = upsert_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
        // 5. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        
    
        let current_time = ic_cdk::api::time() / 1_000_000; // Convert from ns to ms

        let id = upsert_request.id;
        let mut existing_permission = match SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| 
            permissions.borrow().get(&id).cloned()
        ) {
            Some(permission) => permission,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        };
    
        // 6. Handle update vs create based on ID presence
        // UPDATE case
        let has_table_permission = check_permissions_table_access(&requester_api_key.user_id, SystemPermissionType::Edit, is_owner);
        if !is_owner && !has_system_manage_permission(&requester_api_key.user_id, &existing_permission.resource_id) &&!has_table_permission {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to modify system permissions".to_string()).encode()
            );
        }
        

        let prestate = snapshot_prestate();

        // Update modifiable fields
        existing_permission.permission_types = upsert_request.permission_types
                                                    .into_iter()
                                                    .collect::<HashSet<_>>()
                                                    .into_iter()
                                                    .collect();
        existing_permission.begin_date_ms = upsert_request.begin_date_ms.unwrap_or(0);
        existing_permission.expiry_date_ms = upsert_request.expiry_date_ms.unwrap_or(-1);
        existing_permission.note = upsert_request.note.unwrap_or_default();
        existing_permission.last_modified_at = current_time;

        if upsert_request.metadata.is_some() {
            existing_permission.metadata = upsert_request.metadata;
        }

        if let Some(external_id) = upsert_request.external_id.clone() {
            let old_external_id = existing_permission.external_id.clone();
            let new_external_id = Some(ExternalID(external_id.clone()));
            existing_permission.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(existing_permission.id.to_string())
            );
        }
        if let Some(external_payload) = upsert_request.external_payload.clone() {
            existing_permission.external_payload = Some(ExternalPayload(external_payload));
        }

        // Update state
        SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(id.clone(), existing_permission.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update System Permission {}", 
                requester_api_key.user_id,
                id.0
            ).to_string()
        ));

        let final_permission = UpdateSystemPermissionsResponseData {
            permission: existing_permission.cast_fe(&requester_api_key.user_id.clone())
        };

        create_response(
            StatusCode::OK,
            UpdateSystemPermissionsResponse::ok(&final_permission).encode()
        )
    
    }

    pub async fn delete_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // 1. Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
    
        // 2. Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteSystemPermissionRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(e) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
        // 3. Check if permission exists and get it
        let permission = SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow().get(&delete_request.permission_id).cloned()
        });
    
        let permission = match permission {
            Some(p) => p,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::err(404, "Permission not found".to_string()).encode()
            ),
        };

        let old_external_id = permission.external_id.clone();
        let old_internal_id = permission.id.clone().to_string();
    
        // 4. Check authorization
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        let is_granter = permission.granted_by == requester_api_key.user_id;
        let has_table_permission = check_permissions_table_access(&requester_api_key.user_id, SystemPermissionType::Delete, is_owner);
    
        if !is_owner && !is_granter && !has_table_permission {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to delete this permission".to_string()).encode()
            );
        }
    
        let prestate = snapshot_prestate();

        // 5. Delete the permission from all indices
        // Remove from SYSTEM_PERMISSIONS_BY_ID_HASHTABLE
        {SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().remove(&delete_request.permission_id);
        });}
    
        debug_log!("Delete request resource_id {:?}", permission.resource_id.clone());
        // Remove from SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE
        {SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            let mut perms = permissions_by_resource.borrow_mut();
            if let Some(permission_vec) = perms.get_mut(&permission.resource_id) {
                *permission_vec = permission_vec.iter()
                    .filter(|id| **id != delete_request.permission_id)
                    .cloned()
                    .collect();
                
                // Check if empty and remove it
                if permission_vec.is_empty() {
                    perms.remove(&permission.resource_id);
                }
            }
        });}
    
        // Remove from SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE
        {SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            if let Some(permission_vec) = grantee_permissions.borrow_mut().get_mut(&permission.granted_to) {
                *permission_vec = permission_vec.iter().filter(|id| **id != delete_request.permission_id).cloned().collect();
                // // If set is empty, remove the grantee entry (this panicks on error, already borrowed)
                // if permission_vec.is_empty() {
                //     grantee_permissions.borrow_mut().remove(&permission.granted_to);
                // }
            }
        });}
    
        // Remove from SYSTEM_PERMISSIONS_BY_TIME_LIST
        {
            SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|permissions_by_time| {
                let mut list = permissions_by_time.borrow_mut();
                if let Some(pos) = list.iter().position(|id| *id == delete_request.permission_id) {
                    list.remove(pos);
                }
            });
        }

        update_external_id_mapping(
            old_external_id,
            None,
            Some(old_internal_id),
        );

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete System Permission {}", 
                requester_api_key.user_id,
                delete_request.permission_id.0
            ).to_string()
        ));

        

        let final_permission = DeleteSystemPermissionResponseData {
            deleted_id: delete_request.permission_id,
        };

        create_response(
            StatusCode::OK,
            DeleteSystemPermissionResponse::ok(&final_permission).encode()
        )
    }

    pub async fn check_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
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
        let check_request = match serde_json::from_slice::<SystemPermissionCheckRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
    
        if let Err(e) = check_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
        // 3. Parse resource_id into SystemResourceID
        // Clone the value to avoid move issues
        let resource_id_str = check_request.resource_id.clone();
        let resource_id = match resource_id_str.split_once('_') {
            Some(("TABLE", table_name)) => {
                match table_name {
                    "DRIVES" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Drives),
                    "DISKS" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Disks),
                    "CONTACTS" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Contacts),
                    "GROUPS" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Groups),
                    "WEBHOOKS" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Webhooks),
                    "API_KEYS" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Api_Keys),
                    "PERMISSIONS" => SystemResourceID::Table(crate::core::state::permissions::types::SystemTableEnum::Permissions),
                    _ => return create_response(
                        StatusCode::BAD_REQUEST,
                        ErrorResponse::err(400, "Invalid table name".to_string()).encode()
                    ),
                }
            },
            Some(_) => SystemResourceID::Record(SystemRecordIDEnum::Unknown(resource_id_str)),
            None => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid resource ID format".to_string()).encode()
            ),
        };
    
        // 4. Parse grantee_id - Clone it to avoid move issues
        let grantee_id_str = check_request.grantee_id.clone();
        let grantee_id = match parse_permission_grantee_id(&grantee_id_str) {
            Ok(id) => id,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid grantee ID format".to_string()).encode()
            ),
        };
    
        // 5. Check if requester is authorized to check these permissions
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        let is_authorized = if is_owner {
            true
        } else {
            match &grantee_id {
                PermissionGranteeID::User(user_id) if user_id.0 == requester_api_key.user_id.0 => true,
                PermissionGranteeID::Group(group_id) => {
                    is_group_admin(&requester_api_key.user_id, group_id) && 
                    is_user_on_group(&UserID(grantee_id.to_string()), group_id).await
                },
                _ => {
                    has_system_manage_permission(&requester_api_key.user_id, &resource_id) || 
                    check_permissions_table_access(&requester_api_key.user_id, SystemPermissionType::View, is_owner)
                }
            }
        };
    
        if !is_authorized {
            return create_response(
                StatusCode::FORBIDDEN,
                ErrorResponse::err(403, "Not authorized to check permissions for this grantee".to_string()).encode()
            );
        }
    
        // 6. Check permissions
        let mut permissions = HashSet::new();
        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|permissions_by_resource| {
            if let Some(permission_ids) = permissions_by_resource.borrow().get(&resource_id) {
                SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions_by_id| {
                    let permissions_map = permissions_by_id.borrow();
                    for permission_id in permission_ids {
                        if let Some(permission) = permissions_map.get(permission_id) {
                            // Skip if permission is expired or not yet active
                            let current_time = ic_cdk::api::time() as i64 / 1_000_000;
                            if permission.expiry_date_ms > 0 && permission.expiry_date_ms <= current_time {
                                continue;
                            }
                            if permission.begin_date_ms > 0 && permission.begin_date_ms > current_time {
                                continue;
                            }
    
                            // Check if permission applies to this grantee
                            if permission.granted_to == grantee_id {
                                permissions.extend(permission.permission_types.clone());
                            }
                        }
                    }
                });
            }
        });
    
        // Create the response using the wrapper pattern
        let response_data = CheckSystemPermissionResult {
            resource_id: check_request.resource_id,
            grantee_id: check_request.grantee_id,
            permissions: permissions.into_iter().collect(),
        };
    
        create_response(
            StatusCode::OK,
            CheckSystemPermissionResponse::ok(&response_data).encode()
        )
    }

    pub async fn redeem_system_permissions_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };
        // 1. Parse request body
        let body: &[u8] = request.body();
        let redeem_request = match serde_json::from_slice::<RedeemSystemPermissionRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };
        
        if let Err(e) = redeem_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, e.message).encode()
            );
        }
    
        // 2. Convert permission_id string to SystemPermissionID
        let permission_id = SystemPermissionID(redeem_request.permission_id);
    
        // 3. Get existing permission
        let mut permission = match SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
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
        let new_grantee = match parse_permission_grantee_id(&redeem_request.user_id) {
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

        let prestate = snapshot_prestate();
    
        // 6. Update permission and state
        let old_grantee = permission.granted_to.clone();
        permission.granted_to = new_grantee.clone();
        permission.last_modified_at = ic_cdk::api::time() / 1_000_000;
    
        // Update all state tables
        SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
            permissions.borrow_mut().insert(permission_id.clone(), permission.clone());
        });
    
        // Update grantee permissions - remove old, add new
        SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_permissions| {
            let mut table = grantee_permissions.borrow_mut();
            // Remove from old grantee's set
            table.remove(&old_grantee);
            // Add to new grantee's set
            table.entry(new_grantee)
                .or_insert_with(Vec::new)
                .push(permission_id.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Redeem System Permission {}", 
                requester_api_key.user_id,
                permission_id.0
            ).to_string()
        ));


        let final_permission = RedeemSystemPermissionResponseData {
            permission: permission.cast_fe(&requester_api_key.user_id.clone())
        };

        create_response(
            StatusCode::OK,
            RedeemSystemPermissionResponse::ok(&final_permission).encode()
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