pub mod job_runs_handlers {
    use crate::{
        core::{
            api::{
                permissions::system::check_system_permissions,
                replay::diff::{snapshot_poststate, snapshot_prestate},
                uuid::{generate_uuidv4, mark_claimed_uuid},
            },
            state::{
                job_runs::{
                    state::state::{
                        add_job_run_to_vendor_list, remove_job_run_from_vendor_list, JOB_RUNS_BY_ID_HASHTABLE, JOB_RUNS_BY_TIME_LIST, JOB_RUNS_BY_TIME_MEMORY_ID, JOB_RUNS_BY_VENDOR_ID_HASHTABLE
                    },
                    types::{JobRun, JobRunID, JobRunStatus},
                },
                permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum},
                
            },
            types::{IDPrefix, UserID},
        }, debug_log, rest::{
            auth::{authenticate_request, create_auth_error_response},
            job_runs::types::{
                CreateJobRunRequestBody, CreateJobRunResponse, DeleteJobRunRequest, DeleteJobRunResponse,
                DeletedJobRunData, ErrorResponse, GetJobRunResponse, ListJobRunsRequestBody,
                ListJobRunsResponse, ListJobRunsResponseData, UpdateJobRunRequestBody, UpdateJobRunResponse,
            },
            webhooks::types::SortDirection,
        }, MEMORY_MANAGER
    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use ic_stable_structures::StableVec;
    use matchit::Params;
    use serde::Deserialize;

    /// Handles GET requests for a single JobRun by its ID.
    pub async fn get_job_run_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let job_run_id = JobRunID(params.get("job_run_id").unwrap().to_string());

        let job_run = JOB_RUNS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&job_run_id).map(|d| d.clone())
        });

        // Permissions check: User is vendor of the job OR has table view OR has record view.
        let is_vendor_of_job = job_run.as_ref().map_or(false, |jr| requester_api_key.user_id == jr.vendor_id);

        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::JobRuns),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::JobRun(job_run_id.to_string()));
        let record_permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_vendor_of_job && !record_permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) {
            return create_auth_error_response();
        }

        match job_run {
            Some(jr) => {
                create_response(
                    StatusCode::OK,
                    GetJobRunResponse::ok(&jr.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    /// Handles POST requests for listing JobRuns with pagination and filtering.
    pub async fn list_job_runs_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        debug_log!("Handling list_job_runs_handler...");

        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let body = request.body();
        let request_body: ListJobRunsRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(e) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Invalid request format: {}", e)).encode()
            ),
        };

        if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        // Check table-level view permission for general listing access.
        let has_table_view_permission = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::JobRuns),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        ).contains(&SystemPermissionType::View);

        let start_cursor = if let Some(cursor) = request_body.cursor {
            match cursor.parse::<usize>() {
                Ok(idx) => Some(idx),
                Err(_) => return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, "Invalid cursor format".to_string()).encode()
                ),
            }
        } else {
            None
        };

        let total_count = JOB_RUNS_BY_TIME_LIST.with(|list| list.borrow().len()) as usize;

        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListJobRunsResponse::ok(&ListJobRunsResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    direction: request_body.direction,
                    cursor: None,
                }).encode()
            );
        }

        let start_index = if let Some(cursor_idx) = start_cursor {
            cursor_idx.min(total_count.saturating_sub(1))
        } else {
            match request_body.direction {
                SortDirection::Asc => 0,
                SortDirection::Desc => total_count.saturating_sub(1),
            }
        };

        let mut filtered_job_runs = Vec::new();
        let mut end_index = start_index;

        JOB_RUNS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            JOB_RUNS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();

                let mut current_idx = start_index;
                let mut count = 0;

                while count < request_body.page_size && current_idx < total_count {
                    let actual_idx = match request_body.direction {
                        SortDirection::Asc => current_idx,
                        SortDirection::Desc => total_count.saturating_sub(1).saturating_sub(current_idx),
                    };

                    if let Some(job_run_id) = time_index.get(actual_idx as u64) {
                        if let Some(job_run) = id_store.get(&job_run_id) {
                            let is_vendor_of_job = requester_api_key.user_id == job_run.vendor_id;

                            // Check if user has permission to view this specific job run
                            let can_view = has_table_view_permission || is_vendor_of_job || {
                                let resource_id = SystemResourceID::Record(SystemRecordIDEnum::JobRun(job_run.id.to_string()));
                                check_system_permissions(
                                    resource_id,
                                    PermissionGranteeID::User(requester_api_key.user_id.clone())
                                ).contains(&SystemPermissionType::View)
                            };

                            if can_view && request_body.filters.is_empty() { // Placeholder for filters
                                filtered_job_runs.push(job_run.clone());
                            }
                        }
                    }

                    count += 1;
                    if request_body.direction == SortDirection::Asc {
                        current_idx += 1;
                        if current_idx >= total_count { break; }
                    } else {
                        if current_idx == 0 { break; }
                        current_idx -= 1;
                    }
                }
                end_index = current_idx;
            });
        });

        let next_cursor = if filtered_job_runs.len() >= request_body.page_size {
            match request_body.direction {
                SortDirection::Desc => {
                    if end_index > 0 {
                        Some(end_index.to_string())
                    } else {
                        None
                    }
                },
                SortDirection::Asc => {
                    if end_index < total_count {
                        Some(end_index.to_string())
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };

        let total_count_to_return = if has_table_view_permission {
            total_count
        } else {
            filtered_job_runs.len()
        };

        create_response(
            StatusCode::OK,
            ListJobRunsResponse::ok(&ListJobRunsResponseData {
                items: filtered_job_runs.clone().into_iter().map(|jr| jr.cast_fe(&requester_api_key.user_id)).collect(),
                page_size: filtered_job_runs.len(),
                total: total_count_to_return,
                direction: request_body.direction,
                cursor: next_cursor,
            }).encode()
        )
    }

    /// Handles POST requests for creating a new JobRun.
    pub async fn create_job_run_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let body: &[u8] = request.body();
        let create_req = match serde_json::from_slice::<CreateJobRunRequestBody>(body) {
            Ok(body) => body,
            Err(e) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Invalid request format: {}", e)).encode()
            ),
        };

        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        // Check create permission
        let has_create_permission = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::JobRuns),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        ).contains(&SystemPermissionType::Create);

        if !has_create_permission {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();

        let job_run_id = match create_req.id {
            Some(id) => JobRunID(id.to_string()),
            None => JobRunID(generate_uuidv4(IDPrefix::JobRun)),
        };

        let current_time = ic_cdk::api::time() / 1_000_000;

        let job_run = JobRun {
            id: job_run_id.clone(),
            template_id: create_req.template_id,
            vendor_name: create_req.vendor_name.unwrap_or("".to_string()),
            vendor_id: create_req.vendor_id.unwrap_or(UserID("".to_string())),
            status: create_req.status.unwrap_or(JobRunStatus::Requested),
            description: create_req.description.unwrap_or("".to_string()),
            about_url: create_req.about_url.unwrap_or("".to_string()),
            run_url: create_req.run_url.unwrap_or("".to_string()),
            billing_url: create_req.billing_url.unwrap_or("".to_string()),
            support_url: create_req.support_url.unwrap_or("".to_string()),
            delivery_url: create_req.delivery_url.unwrap_or("".to_string()),
            verification_url: create_req.verification_url.unwrap_or("".to_string()),
            installation_url: create_req.installation_url.unwrap_or("".to_string()),
            title: create_req.title,
            subtitle: create_req.subtitle.unwrap_or("".to_string()),
            pricing: create_req.pricing.unwrap_or("".to_string()),
            vendor_notes: create_req.vendor_notes.unwrap_or("".to_string()),
            notes: create_req.notes.unwrap_or("".to_string()),
            created_at: current_time,
            updated_at: current_time,
            last_updated_at: current_time,
            related_resources: create_req.related_resources.unwrap_or(vec![]),
            tracer: create_req.tracer,
            labels: create_req.labels.unwrap_or(vec![]),
            external_id: create_req.external_id,
            external_payload: create_req.external_payload,
        };

        JOB_RUNS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(job_run_id.clone(), job_run.clone());
        });

        JOB_RUNS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(&job_run_id.clone());
        });

        // Add to the vendor-specific list
        add_job_run_to_vendor_list(&job_run.vendor_id, &job_run_id);

        mark_claimed_uuid(&job_run_id.clone().to_string());

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create JobRun {}",
                requester_api_key.user_id,
                job_run_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            CreateJobRunResponse::ok(&job_run.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    /// Handles POST requests for updating an existing JobRun.
    pub async fn update_job_run_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let body: &[u8] = request.body();
        let update_req = match serde_json::from_slice::<UpdateJobRunRequestBody>(body) {
            Ok(body) => body,
            Err(e) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, format!("Invalid request format: {}", e)).encode()
            ),
        };

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        let job_run_id = JobRunID(update_req.id);

        let mut job_run = match JOB_RUNS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&job_run_id).map(|d| d.clone())) {
            Some(jr) => jr,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Permissions check: User is vendor of the job OR has table edit OR has record edit.
        let is_vendor_of_job = requester_api_key.user_id == job_run.vendor_id;

        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::JobRuns),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::JobRun(job_run_id.to_string()));
        let record_permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_vendor_of_job && !record_permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
            return create_auth_error_response();
        }

        let prestate = snapshot_prestate();
        let current_time = ic_cdk::api::time() / 1_000_000;

        // Update fields (only those allowed to be updated)
        if let Some(title) = update_req.title {
            job_run.title = title;
        }
        if let Some(vendor_name) = update_req.vendor_name {
            job_run.vendor_name = vendor_name;
        }
        if let Some(vendor_id) = update_req.vendor_id {
            job_run.vendor_id = vendor_id;
        }
        if let Some(description) = update_req.description {
            job_run.description = description;
        }
        if let Some(notes) = update_req.notes {
            job_run.notes = notes;
        }
        
        if let Some(template_id) = update_req.template_id {
            job_run.template_id = Some(template_id);
        }
        if let Some(status) = update_req.status {
            job_run.status = status;
        }
        if let Some(about_url) = update_req.about_url {
            job_run.about_url = about_url;
        }
        if let Some(run_url) = update_req.run_url {
            job_run.run_url = run_url;
        }
        if let Some(billing_url) = update_req.billing_url {
            job_run.billing_url = billing_url;
        }
        if let Some(support_url) = update_req.support_url {
            job_run.support_url = support_url;
        }
        if let Some(delivery_url) = update_req.delivery_url {
            job_run.delivery_url = delivery_url;
        }
        if let Some(verification_url) = update_req.verification_url {
            job_run.verification_url = verification_url;
        }
        if let Some(installation_url) = update_req.installation_url {
            job_run.installation_url = installation_url;
        }
        if let Some(subtitle) = update_req.subtitle {
            job_run.subtitle = subtitle;
        }
        if let Some(pricing) = update_req.pricing {
            job_run.pricing = pricing;
        }
        if let Some(vendor_notes) = update_req.vendor_notes {
            job_run.vendor_notes = vendor_notes;
        }
        if let Some(related_resources) = update_req.related_resources {
            job_run.related_resources = related_resources;
        }
        if let Some(tracer) = update_req.tracer {
            job_run.tracer = Some(tracer);
        }
        if let Some(labels) = update_req.labels {
            job_run.labels = labels;
        }
        if let Some(external_id) = update_req.external_id {
            job_run.external_id = Some(external_id);
        }
        if let Some(external_payload) = update_req.external_payload {
            job_run.external_payload = Some(external_payload);
        }

        job_run.updated_at = current_time;
        job_run.last_updated_at = current_time;

        JOB_RUNS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(job_run_id.clone(), job_run.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update JobRun {}",
                requester_api_key.user_id,
                job_run_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            UpdateJobRunResponse::ok(&job_run.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    /// Handles POST requests for deleting a JobRun.
    pub async fn delete_job_run_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let prestate = snapshot_prestate();

        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteJobRunRequest>(body) {
            Ok(req) => req,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

        if let Err(validation_error) = delete_request.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        let job_run_id = delete_request.id.clone();

        // Retrieve the job run to get vendor_id before it's removed
        let job_run = JOB_RUNS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&job_run_id).map(|d| d.clone())
        });

        let vendor_id_for_cleanup = job_run.as_ref().map(|jr| jr.vendor_id.clone());

        // Permissions check: User is vendor of the job OR has table delete OR has record delete.
        let is_vendor_of_job = job_run.as_ref().map_or(false, |jr| requester_api_key.user_id == jr.vendor_id);

        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::JobRuns),
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::JobRun(job_run_id.to_string()));
        let record_permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(requester_api_key.user_id.clone())
        );

        if !is_vendor_of_job && !record_permissions.contains(&SystemPermissionType::Delete) && !table_permissions.contains(&SystemPermissionType::Delete) {
            return create_auth_error_response();
        }

        // Remove from main stores
        JOB_RUNS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&job_run_id);
        });

        // Rebuild StableVec for JOB_RUNS_BY_TIME_LIST to remove the item
        JOB_RUNS_BY_TIME_LIST.with(|store| {
            let mut new_vec = StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(JOB_RUNS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize new StableVec");

            let store_ref = store.borrow();
            for i in 0..store_ref.len() {
                if let Some(id) = store_ref.get(i) {
                    if id != job_run_id {
                        new_vec.push(&id);
                    }
                }
            }
            drop(store_ref); // Drop the mutable borrow before replacing
            *store.borrow_mut() = new_vec;
        });

        // Remove from JOB_RUNS_BY_VENDOR_ID_HASHTABLE
        if let Some(vendor_id) = vendor_id_for_cleanup {
            remove_job_run_from_vendor_list(&vendor_id, &job_run_id);
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete JobRun {}",
                requester_api_key.user_id,
                job_run_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            DeleteJobRunResponse::ok(&DeletedJobRunData {
                id: job_run_id,
                deleted: true
            }).encode()
        )
    }

    // This `create_response` function is duplicated from the Disks example.
    // In a real project, you might move this to a shared `src/rest/types.rs` or similar.
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