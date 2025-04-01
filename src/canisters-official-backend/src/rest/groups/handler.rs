// src/rest/groups/handler.rs


pub mod groups_handlers {
    use crate::{
        core::{api::{permissions::{self, system::check_system_permissions}, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}}, state::{drives::{state::state::{update_external_id_mapping, DRIVE_ID, OWNER_ID, URL_ENDPOINT}, types::{DriveID, DriveRESTUrlEndpoint, ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, group_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::GroupInvite}, groups::{state::state::{is_user_on_group, GROUPS_BY_ID_HASHTABLE, GROUPS_BY_TIME_LIST}, types::{Group, GroupID}}}, types::{IDPrefix, PublicKeyICP}}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, groups::types::{CreateGroupRequestBody, CreateGroupResponse, DeleteGroupRequestBody, DeleteGroupResponse, DeletedGroupData, ErrorResponse, GetGroupResponse, ListGroupsRequestBody, ListGroupsResponseData, UpdateGroupRequestBody, UpdateGroupResponse, ValidateGroupRequestBody, ValidateGroupResponse, ValidateGroupResponseData}, types::ApiResponse}
        
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_group_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let id = GroupID(params.get("group_id").unwrap().to_string());

        // Only owner can read groups for now
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        // Check table-level permissions for Groups table
        let permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        let is_member = is_user_on_group(&requester_api_key.user_id, &id).await;

        if !permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) && !is_owner && !is_member {
            return create_auth_error_response();
        }

        let group = GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&id).cloned()
        });

        match group {
            Some(group) => create_response(
                StatusCode::OK,
                GetGroupResponse::ok(&group.cast_fe(&requester_api_key.user_id)).encode()
            ),
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }


    pub async fn list_groups_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        
        debug_log!("Listing groups...");

        let query: ListGroupsRequestBody = match serde_json::from_slice(request.body()) {
            Ok(q) => q,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_err) = query.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_err.field, validation_err.message)
                ).encode()
            );
        }

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can list groups for now
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        // Check table-level permissions for Groups table
        let permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        debug_log!("Permissions: {:?}", permissions);

        if !permissions.contains(&SystemPermissionType::View) && !is_owner {
            return create_auth_error_response();
        }

        let groups = GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow()
                .values()
                .cloned()
                .collect::<Vec<_>>()
        });

        let response_data = ListGroupsResponseData {
            items: groups.clone().into_iter().map(|group| {
                group.cast_fe(&requester_api_key.user_id)
            }).collect(),
            page_size: 50, // Using the default page size
            total: groups.len(),
            cursor_up: None,
            cursor_down: None,
        };
    
        // Wrap it in a ApiResponse and encode
        create_response(
            StatusCode::OK,
            ApiResponse::ok(&response_data).encode()
        )

    }

    pub async fn create_group_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can create/update groups for now
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateGroupRequestBody>(body).unwrap();
        
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        // Check table-level permissions for Groups table
        let permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !permissions.contains(&SystemPermissionType::Create) && !is_owner {
            return create_auth_error_response();
        }
        let prestate = snapshot_prestate();
        
        let now = ic_cdk::api::time();

        let group_id = match create_req.id {
            Some(id) => GroupID(id.to_string()),
            None => GroupID(generate_uuidv4(IDPrefix::Group)),
        };

        // Create new group
        let new_group = Group {
            id: group_id.clone(),
            name: create_req.name,
            avatar: create_req.avatar,
            owner: requester_api_key.user_id.clone(),
            private_note: create_req.private_note,
            public_note: create_req.public_note,
            admin_invites: Vec::new(),
            member_invites: Vec::new(),
            created_at: now,
            last_modified_at: now,
            drive_id: DRIVE_ID.with(|id| id.clone()),
            endpoint_url: DriveRESTUrlEndpoint(
                create_req.endpoint_url
                    .unwrap_or(URL_ENDPOINT.with(|url| url.borrow().clone()).0)
                    .trim_end_matches('/')
                    .to_string()
            ),
            labels: vec![],
            external_id: Some(ExternalID(create_req.external_id.unwrap_or("".to_string()))),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
        };
        update_external_id_mapping(None, new_group.external_id.clone(), Some(new_group.id.clone().to_string()));

        // Update state
        GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(group_id.clone(), new_group.clone());
        });

        GROUPS_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(group_id.clone());
        });

        mark_claimed_uuid(&group_id.clone().to_string());

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Group {}", 
                requester_api_key.user_id,
                group_id.0
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            CreateGroupResponse::ok(&new_group.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn update_group_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can create/update groups for now
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());

        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateGroupRequestBody>(body).unwrap();
        
        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }

        // Check table-level permissions for Groups table
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
        let permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Group(update_req.id.clone())),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) && !is_owner {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();

        let group_id = GroupID(update_req.id);
        
        // Get existing group
        let mut group = match GROUPS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&group_id).cloned()) {
            Some(group) => group,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Update fields
        if let Some(name) = update_req.name {
            group.name = name;
        }
        if let Some(avatar) = update_req.avatar {
            group.avatar = Some(avatar);
        }
        if let Some(public_note) = update_req.public_note {
            group.public_note = Some(public_note);
        }
        if let Some(private_note) = update_req.private_note {
            group.private_note = Some(private_note);
        }
        if let Some(endpoint_url) = update_req.endpoint_url {
            group.endpoint_url = DriveRESTUrlEndpoint(endpoint_url.trim_end_matches('/')
            .to_string());
        }
        group.last_modified_at = ic_cdk::api::time();

        if let Some(external_id) = update_req.external_id.clone() {
            let old_external_id = group.external_id.clone();
            let new_external_id = Some(ExternalID(external_id.clone()));
            group.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(group.id.to_string())
            );
        }
        if let Some(external_payload) = update_req.external_payload.clone() {
            group.external_payload = Some(ExternalPayload(external_payload));
        }

        // Update state
        GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(group.id.clone(), group.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Group {}", 
                requester_api_key.user_id,
                group_id.0
            ).to_string()
        ));

        create_response(
            StatusCode::OK,
            UpdateGroupResponse::ok(&group.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn delete_group_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteGroupRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400, 
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }
    
        let group_id = GroupID(delete_request.id.clone());
    
        // Only owner can delete groups for now
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow());
        // Check table-level permissions for Groups table
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
        let permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Group(group_id.clone().to_string())),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if  !permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) && !is_owner {
            return create_auth_error_response();
        }
        

        let prestate = snapshot_prestate();
        
    
        // Get group to verify it exists
        let group = match GROUPS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&group_id).cloned()) {
            Some(group) => group,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };
        let old_external_id = group.external_id.clone();
        let old_internal_id = Some(group_id.clone().to_string());
    
    
        // First, get all invites to know which users we need to update
        let invites_to_remove = INVITES_BY_ID_HASHTABLE.with(|store| {
            let store = store.borrow();
            group.member_invites.clone().iter()
                .filter_map(|invite_id| store.get(invite_id).cloned())
                .collect::<Vec<GroupInvite>>()
        });
    
        // Remove invites from INVITES_BY_ID_HASHTABLE
        INVITES_BY_ID_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            for invite_id in &group.member_invites {
                store.remove(invite_id);
            }
        });
    
        // Remove invites from USERS_INVITES_LIST_HASHTABLE
        USERS_INVITES_LIST_HASHTABLE.with(|store| {
            let mut store = store.borrow_mut();
            // For each invite we're removing, update the corresponding user's invite list
            for invite in &invites_to_remove {
                if let Some(user_invites) = store.get_mut(&invite.invitee_id) {
                    user_invites.retain(|id| !group.member_invites.contains(id));
                }
            }
        });
    
        // Remove group from GROUPS_BY_ID_HASHTABLE
        GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&group_id);
        });
    
        // Remove group from GROUPS_BY_TIME_LIST
        GROUPS_BY_TIME_LIST.with(|list| {
            let mut list = list.borrow_mut();
            if let Some(pos) = list.iter().position(|id| *id == group_id) {
                list.remove(pos);
            }
        });

        update_external_id_mapping(old_external_id, None, old_internal_id);

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Group {}", 
                requester_api_key.user_id,
                group_id.0
            ).to_string()
        ));
    
        create_response(
            StatusCode::OK,
            DeleteGroupResponse::ok(&DeletedGroupData {
                id: delete_request.id,
                deleted: true
            }).encode()
        )
    }

    pub async fn validate_group_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Parse request body
        let body: &[u8] = request.body();
        let validate_request = match serde_json::from_slice::<ValidateGroupRequestBody>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = validate_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(
                    400,
                    format!("Validation error: {} - {}", validation_error.field, validation_error.message)
                ).encode()
            );
        }
        
        // Get group to verify it exists
        let group_exists = GROUPS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().contains_key(&validate_request.group_id)
        });
    
        if !group_exists {
            return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            );
        }
    
        // Use existing is_user_on_group function to check membership
        let is_member = is_user_on_group(&validate_request.user_id, &validate_request.group_id).await;
    
        let response_data = ValidateGroupResponseData {
            is_member,
            group_id: validate_request.group_id,
            user_id: validate_request.user_id
        };
    
        if is_member {
            create_response(
                StatusCode::OK,
                ValidateGroupResponse::ok(&response_data).encode()
            )
        } else {
            create_response(
                StatusCode::FORBIDDEN,
                ApiResponse::<ValidateGroupResponseData>::err(403, "User is not a member of this group".to_string()).encode()
            )
        }
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