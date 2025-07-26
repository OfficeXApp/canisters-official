Please help me add this JobRun state & routes to my rust icp canister code. please give me the files required, using disks as reference. please strictly follow what you see in the Disk reference, including those imports.

note that aside from the basic JOB_RUN_BY_ID a,d JOB_RUN_BY_TIME_LIST, there should also be a JOB_RUN_LIST_BY_VENDOR_ID, and our handlers should remember to add and remove from this map.

### Concept of JobRun

export interface JobRun {
id: JobRunID;
template_id?: string; // no guarnatees on this, only set on create
vendor_name: string; // cannot be updated, only set on create
vendor_id: UserID; // cannot be updated, only set on create
status: JobRunStatus; // can be updated by vendor
description: string; // cannot be updated, only set on create
billing_url: string; // can be updated by vendor
support_url: string; // can be updated by vendor
delivery_url: string; // can be updated by vendor
verification_url: string; // can be updated by vendor
installation_url: string; // the script to run to install the job
title: string; // cannot be updated, only set on create
subtitle: string; // can be updated
pricing: string; // can be updated
vendor_notes: string; // can be updated by vendor
notes: string; // cannot be viewed or updated by vendor
created_at: number;
updated_at: number;
last_updated_at: number;
related_resources: string[]; // list of ID strings, can be updated
tracer?: string; // can be updated by vendor
}

### Reference "Disks" State & Types

// src/core/state/disks/state.rs
pub mod state {
use std::cell::RefCell;
use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, BTreeMap, DefaultMemoryImpl, StableBTreeMap, StableVec};

    use crate::{core::{api::uuid::generate_uuidv4, state::{directory::{state::state::{folder_uuid_to_metadata, full_folder_path_to_uuid}, types::{DriveFullFilePath, FolderID, FolderRecord}}, disks::types::{Disk, DiskID, DiskTypeEnum}, drives::{state::state::{DRIVE_ID, OWNER_ID}, types::{DriveID, ExternalID}}}, types::{IDPrefix, UserID}}, debug_log, MEMORY_MANAGER};

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    pub const DISKS_MEMORY_ID: MemoryId = MemoryId::new(11);
    pub const DISKS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(12);

    thread_local! {
        // Replace HashMap with StableBTreeMap for disks by ID
        pub(crate) static DISKS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<DiskID, Disk, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DISKS_MEMORY_ID))
            )
        );

        // Replace Vec with StableVec for disks by time list
        pub(crate) static DISKS_BY_TIME_LIST: RefCell<StableVec<DiskID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DISKS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize DISKS_BY_TIME_LIST")
        );
    }

    pub fn initialize() {
        // Force thread_locals in this module to initialize
        DISKS_BY_ID_HASHTABLE.with(|_| {});
        DISKS_BY_TIME_LIST.with(|_| {});
    }

}

use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
// src/core/state/disks/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use std::{borrow::Cow, fmt};

use crate::{core::{api::permissions::system::check_system_permissions, state::{directory::types::FolderID, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, labels::types::{redact_label, LabelStringValue}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::UserID}, rest::{disks::types::DiskFE, labels::types::LabelFE}};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct DiskID(pub String);
impl fmt::Display for DiskID {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
write!(f, "{}", self.0)
}
}

impl Storable for DiskID {
const BOUND: Bound = Bound::Bounded {
max_size: 256, // Adjust based on your needs
is_fixed_size: false,
};

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize DiskID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize DiskID")
    }

}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub struct Disk {
pub id: DiskID,
pub name: String,
pub disk_type: DiskTypeEnum,
pub private_note: Option<String>,
pub public_note: Option<String>,
pub auth_json: Option<String>,
pub labels: Vec<LabelStringValue>,
pub created_at: u64,
pub root_folder: FolderID,
pub trash_folder: FolderID,
pub external_id: Option<ExternalID>,
pub external_payload: Option<ExternalPayload>,
pub endpoint: Option<String>,
}

impl Storable for Disk {
const BOUND: Bound = Bound::Bounded {
max_size: 256 \* 256, // Adjust based on your needs
is_fixed_size: false,
};

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize Disk");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize Disk")
    }

}

impl Disk {
pub fn cast_fe(&self, user_id: &UserID) -> DiskFE {
let disk = self.clone();

        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Disk(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Disks),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        DiskFE {
            disk,
            permission_previews
        }.redacted(user_id)
    }

}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, SerdeDiff, CandidType)] #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiskTypeEnum {
BrowserCache,
LocalSsd,
AwsBucket,
StorjWeb3,
IcpCanister,
}
impl fmt::Display for DiskTypeEnum {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
match self {
DiskTypeEnum::BrowserCache => write!(f, "BROWSER_CACHE"),
DiskTypeEnum::LocalSsd => write!(f, "LOCAL_SSD"),
DiskTypeEnum::AwsBucket => write!(f, "AWS_BUCKET"),
DiskTypeEnum::StorjWeb3 => write!(f, "STORJ_WEB3"),
DiskTypeEnum::IcpCanister => write!(f, "ICP_CANISTER"),
}
}
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct AwsBucketAuth {
pub(crate) endpoint: String,
pub(crate) access_key: String,
pub(crate) secret_key: String,
pub(crate) bucket: String,
pub(crate) region: String,  
}

### Reference "Disks" Route & Types

// src/rest/disks/route.rs
use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const DISKS_GET_PATH: &str = genroute!("/disks/get/{disk_id}");
pub const DISKS_LIST_PATH: &str = genroute!("/disks/list");
pub const DISKS_CREATE_PATH: &str = genroute!("/disks/create");
pub const DISKS_UPDATE_PATH: &str = genroute!("/disks/update");
pub const DISKS_DELETE_PATH: &str = genroute!("/disks/delete");

type HandlerEntry = (&'static str, &'static str, RouteHandler);

pub fn init_routes() {
let routes: &[HandlerEntry] = &[
(
"GET",
DISKS_GET_PATH,
|req, params| Box::pin(crate::rest::disks::handler::disks_handlers::get_disk_handler(req, params)),
),
(
"POST",
DISKS_LIST_PATH,
|req, params| Box::pin(crate::rest::disks::handler::disks_handlers::list_disks_handler(req, params)),
),
(
"POST",
DISKS_CREATE_PATH,
|req, params| Box::pin(crate::rest::disks::handler::disks_handlers::create_disk_handler(req, params)),
),
(
"POST",
DISKS_UPDATE_PATH,
|req, params| Box::pin(crate::rest::disks::handler::disks_handlers::update_disk_handler(req, params)),
),
(
"POST",
DISKS_DELETE_PATH,
|req, params| Box::pin(crate::rest::disks::handler::disks_handlers::delete_disk_handler(req, params)),
)
];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }

}

// src/rest/disks/types.rs

use serde::{Deserialize, Serialize};

use crate::{
core::{api::permissions::system::check_system_permissions, state::{disks::types::{Disk, DiskID, DiskTypeEnum}, drives::state::state::OWNER_ID, labels::{state::validate_uuid4_string_with_prefix, types::redact_label}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::{ClientSuggestedUUID, IDPrefix, UserID}},
rest::{types::{validate_external_id, validate_external_payload, validate_id_string, validate_short_string, validate_unclaimed_uuid, validate_url, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskFE { #[serde(flatten)]
pub disk: Disk,
pub permission_previews: Vec<SystemPermissionType>,
}

impl DiskFE {
pub fn redacted(&self, user_id: &UserID) -> Self {
let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.disk.auth_json = None;
                redacted.disk.private_note = None;
            }
        }
        // Filter labels
        redacted.disk.labels = match is_owner {
            true => redacted.disk.labels,
            false => redacted.disk.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };

        redacted
    }

}

#[derive(Debug, Clone, Deserialize)]
pub struct ListDisksRequestBody { #[serde(default)]
pub filters: String, #[serde(default = "default_page_size")]
pub page_size: usize, #[serde(default)]
pub direction: SortDirection,
pub cursor: Option<String>,
}

fn default_page_size() -> usize {
50
}

impl ListDisksRequestBody {
pub fn validate_body(&self) -> Result<(), ValidationError> {
// Validate filters string length
if self.filters.len() > 256 {
return Err(ValidationError {
field: "filters".to_string(),
message: "Filters must be 256 characters or less".to_string(),
});
}

        // Validate page_size is reasonable
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(ValidationError {
                field: "page_size".to_string(),
                message: "Page size must be between 1 and 1000".to_string(),
            });
        }

        // Validate cursor strings if present
        if let Some(cursor) = &self.cursor {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }

}

#[derive(Debug, Clone, Serialize)]
pub struct ListDisksResponseData {
pub items: Vec<DiskFE>,
pub page_size: usize,
pub total: usize,
pub direction: SortDirection,
pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)] #[serde(deny_unknown_fields)]
pub struct CreateDiskRequestBody {
pub id: Option<ClientSuggestedUUID>,
pub name: String,
pub disk_type: DiskTypeEnum,
pub public_note: Option<String>,
pub private_note: Option<String>,
pub auth_json: Option<String>,
pub external_id: Option<String>,
pub external_payload: Option<String>,
pub endpoint: Option<String>,
}
impl CreateDiskRequestBody {
pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::Disk)?;
        }

        // Validate name (up to 256 chars)
        validate_short_string(&self.name, "name")?;

        // Validate public_note if provided (up to 8192 chars for descriptions)
        if let Some(public_note) = &self.public_note {
            if public_note.len() > 8192 {
                return Err(ValidationError {
                    field: "public_note".to_string(),
                    message: "Public note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate private_note if provided (up to 8192 chars for descriptions)
        if let Some(private_note) = &self.private_note {
            if private_note.len() > 8192 {
                return Err(ValidationError {
                    field: "private_note".to_string(),
                    message: "Private note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate auth_json if provided (up to 8192 chars for large JSON payload)
        if let Some(auth_json) = &self.auth_json {
            if auth_json.len() > 8192 {
                return Err(ValidationError {
                    field: "auth_json".to_string(),
                    message: "Auth JSON must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }

        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }

        if let Some(endpoint) = &self.endpoint {
            validate_url(endpoint, "endpoint")?;
        }

        Ok(())
    }

}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDiskRequestBody {
pub id: String, #[serde(skip_serializing_if = "Option::is_none")]
pub name: Option<String>, #[serde(skip_serializing_if = "Option::is_none")]
pub public_note: Option<String>, #[serde(skip_serializing_if = "Option::is_none")]
pub private_note: Option<String>, #[serde(skip_serializing_if = "Option::is_none")]
pub auth_json: Option<String>, #[serde(skip_serializing_if = "Option::is_none")]
pub external_id: Option<String>, #[serde(skip_serializing_if = "Option::is_none")]
pub external_payload: Option<String>, #[serde(skip_serializing_if = "Option::is_none")]
pub endpoint: Option<String>,
}
impl UpdateDiskRequestBody {
pub fn validate_body(&self) -> Result<(), ValidationError> {
// Validate ID string
validate_id_string(&self.id, "id")?;

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_short_string(name, "name")?;
        }

        // Validate public_note if provided
        if let Some(public_note) = &self.public_note {
            if public_note.len() > 8192 {
                return Err(ValidationError {
                    field: "public_note".to_string(),
                    message: "Public note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate private_note if provided
        if let Some(private_note) = &self.private_note {
            if private_note.len() > 8192 {
                return Err(ValidationError {
                    field: "private_note".to_string(),
                    message: "Private note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate auth_json if provided
        if let Some(auth_json) = &self.auth_json {
            if auth_json.len() > 8192 {
                return Err(ValidationError {
                    field: "auth_json".to_string(),
                    message: "Auth JSON must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }

        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }

        Ok(())
    }

}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDiskRequest {
pub id: DiskID,
}
impl DeleteDiskRequest {
pub fn validate_body(&self) -> Result<(), ValidationError> {
// Validate the DiskID
validate_id_string(&self.id.0, "id")?;

        // Check if it starts with the correct prefix
        let disk_prefix = crate::core::types::IDPrefix::Disk.as_str();
        if !self.id.0.starts_with(disk_prefix) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("Disk ID must start with '{}'", disk_prefix),
            });
        }

        Ok(())
    }

}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedDiskData {
pub id: DiskID,
pub deleted: bool,
}

pub type GetDiskResponse<'a> = ApiResponse<'a, DiskFE>;
pub type DeleteDiskResponse<'a> = ApiResponse<'a, DeletedDiskData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ListDisksResponse<'a> = ApiResponse<'a, ListDisksResponseData>;
pub type CreateDiskResponse<'a> = ApiResponse<'a, DiskFE>;
pub type UpdateDiskResponse<'a> = ApiResponse<'a, DiskFE>;

// src/rest/disks/handler.rs

pub mod disks_handlers {
use crate::{
core::{api::{internals::drive_internals::validate_auth_json, permissions::system::check_system_permissions, replay::diff::{snapshot_poststate, snapshot_prestate}, uuid::{generate_uuidv4, mark_claimed_uuid}}, state::{disks::{state::state::{ensure_disk_root_and_trash_folder, DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST, DISKS_BY_TIME_MEMORY_ID}, types::{AwsBucketAuth, Disk, DiskID, DiskTypeEnum}}, drives::{state::state::{update_external_id_mapping, DRIVE_ID, OWNER_ID}, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::IDPrefix}, debug_log, rest::{auth::{authenticate_request, create_auth_error_response}, disks::types::{ CreateDiskRequestBody, CreateDiskResponse, DeleteDiskRequest, DeleteDiskResponse, DeletedDiskData, ErrorResponse, GetDiskResponse, ListDisksRequestBody, ListDisksResponse, ListDisksResponseData, UpdateDiskRequestBody, UpdateDiskResponse}, webhooks::types::SortDirection}, MEMORY_MANAGER

    };
    use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
    use ic_stable_structures::{StableVec};
    use matchit::Params;
    use serde::Deserialize;
    #[derive(Deserialize, Default)]
    struct ListQueryParams {
        title: Option<String>,
        completed: Option<bool>,
    }

    pub async fn get_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Only owner can access disk.private_note
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Get disk ID from params
        let disk_id = DiskID(params.get("disk_id").unwrap().to_string());

        // Get the disk
        let disk = DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&disk_id).map(|d| d.clone())
        });

        // Check permissions if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Disks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            if !permissions.contains(&SystemPermissionType::View) && !table_permissions.contains(&SystemPermissionType::View) {
                return create_auth_error_response();
            }
        }

        match disk {
            Some(mut disk) => {
                create_response(
                    StatusCode::OK,
                    GetDiskResponse::ok(&disk.cast_fe(&requester_api_key.user_id)).encode()
                )
            },
            None => create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        }
    }

    pub async fn list_disks_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {

        debug_log!("Handling list_disks_handler...");

        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        // Check if the requester is the owner
        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Check table-level permissions if not owner
        let has_table_permission = if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Disks);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            permissions.contains(&SystemPermissionType::View)
        } else {
            true
        };

        debug_log!("has_table_permission: {}", has_table_permission);

        // Parse request body
        let body = request.body();
        let request_body: ListDisksRequestBody = match serde_json::from_slice(body) {
            Ok(body) => body,
            Err(_) => return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, "Invalid request format".to_string()).encode()
            ),
        };

         // Validate request body
         if let Err(validation_error) = request_body.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        // Parse cursor if provided
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

        // Get total count
        let total_count = DISKS_BY_TIME_LIST.with(|list| list.borrow().len()) as usize;

        // If there are no disks, return early
        if total_count == 0 {
            return create_response(
                StatusCode::OK,
                ListDisksResponse::ok(&ListDisksResponseData {
                    items: vec![],
                    page_size: 0,
                    total: 0,
                    direction: request_body.direction,
                    cursor: None,
                }).encode()
            );
        }

        // Determine starting point based on cursor
        let start_index = if let Some(cursor_idx) = start_cursor {
            cursor_idx.min(total_count - 1)
        } else {
            match request_body.direction {
                SortDirection::Asc => 0,
                SortDirection::Desc => total_count - 1,
            }
        };

        // Get disks with pagination and filtering, applying permission checks
        let mut filtered_disks = Vec::new();
        let mut processed_count = 0;
        let mut end_index = start_index;  // Track where we ended for cursor calculation
        let mut total_count_to_return = 0; // Will use this for the response

        // If user is owner or has table access, they get the actual total count
        if is_owner || has_table_permission {
            total_count_to_return = total_count;
        }

        DISKS_BY_TIME_LIST.with(|time_index| {
            let time_index = time_index.borrow();
            DISKS_BY_ID_HASHTABLE.with(|id_store| {
                let id_store = id_store.borrow();

                match request_body.direction {
                    SortDirection::Desc => {
                        let mut current_idx = start_index;
                        while filtered_disks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(disk_id) = time_index.get(current_idx as u64) {
                                if let Some(disk) = id_store.get(&disk_id) {
                                    // Check if user has permission to view this specific disk
                                    let can_view = is_owner || has_table_permission || {
                                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk.id.to_string()));
                                        let permissions = check_system_permissions(
                                            resource_id,
                                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                                        );
                                        permissions.contains(&SystemPermissionType::View)
                                    };

                                    if can_view && request_body.filters.is_empty() {
                                        filtered_disks.push(disk.clone());
                                    }
                                }
                            }
                            if current_idx == 0 {
                                break;
                            }
                            current_idx -= 1;
                            processed_count += 1;
                        }
                        end_index = current_idx;
                    },
                    SortDirection::Asc => {
                        let mut current_idx = start_index;
                        while filtered_disks.len() < request_body.page_size && current_idx < total_count {
                            if let Some(disk_id) = time_index.get(current_idx as u64) {
                                if let Some(disk) = id_store.get(&disk_id) {
                                    // Check if user has permission to view this specific disk
                                    let can_view = is_owner || has_table_permission || {
                                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk.id.to_string()));
                                        let permissions = check_system_permissions(
                                            resource_id,
                                            PermissionGranteeID::User(requester_api_key.user_id.clone())
                                        );
                                        permissions.contains(&SystemPermissionType::View)
                                    };

                                    if can_view && request_body.filters.is_empty() {
                                        filtered_disks.push(disk.clone());
                                    }
                                }
                            }
                            current_idx += 1;
                            processed_count += 1;
                            if current_idx >= total_count {
                                break;
                            }
                        }
                        end_index = current_idx - 1;
                    }
                }
            });
        });

        // Calculate next cursor based on direction and where we ended
        let next_cursor = if filtered_disks.len() >= request_body.page_size {
            match request_body.direction {
                SortDirection::Desc => {
                    if end_index > 0 {
                        Some(end_index.to_string())
                    } else {
                        None
                    }
                },
                SortDirection::Asc => {
                    if end_index < total_count - 1 {
                        Some((end_index + 1).to_string())
                    } else {
                        None
                    }
                }
            }
        } else {
            None  // No more results available
        };

        // Determine the total count for the response
        // If the user doesn't have full access and we haven't calculated the total yet,
        // set it to batch size + 1 if there are more results available
        if !is_owner && !has_table_permission {
            if next_cursor.is_some() {
                // If there are more results (next cursor exists), return batch size + 1
                total_count_to_return = filtered_disks.len() + 1;
            } else {
                // Otherwise, just return the batch size
                total_count_to_return = filtered_disks.len();
            }
        }

        create_response(
            StatusCode::OK,
            ListDisksResponse::ok(&ListDisksResponseData {
                items: filtered_disks.clone().into_iter().map(|disk| {
                    disk.cast_fe(&requester_api_key.user_id)
                }).collect(),
                page_size: filtered_disks.len(),
                total: total_count_to_return,
                direction: request_body.direction,
                cursor: next_cursor,
            }).encode()
        )
    }

    pub async fn create_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        // Parse request body
        let body: &[u8] = request.body();
        let create_req = serde_json::from_slice::<CreateDiskRequestBody>(body).unwrap();
        if let Err(validation_error) = create_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        // Check create permission if not owner
        if !is_owner {
            let resource_id = SystemResourceID::Table(SystemTableEnum::Disks);
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            if !permissions.contains(&SystemPermissionType::Create) {
                return create_auth_error_response();
            }
        }

        // Validate that auth_json is provided and valid for AwsBucket or StorjWeb3 types.
        if let Err(err_msg) = validate_auth_json(&create_req.disk_type, &create_req.auth_json) {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, err_msg).encode()
            );
        }
        let prestate = snapshot_prestate();


        // Create new disk
        let disk_id = match create_req.id {
            Some(id) => DiskID(id.to_string()),
            None => DiskID(generate_uuidv4(IDPrefix::Disk)),
        };

        let (root_folder_uuid, trash_folder_uuid) = ensure_disk_root_and_trash_folder(
            &disk_id,
            &requester_api_key.user_id,
            &DRIVE_ID.with(|drive_id| drive_id.clone()),
            create_req.disk_type.clone()
        );

        let new_external_id = Some(ExternalID(create_req.external_id.unwrap_or("".to_string())));
        let disk = Disk {
            id: disk_id.clone(),
            name: create_req.name,
            public_note: create_req.public_note,
            private_note: create_req.private_note,
            auth_json: create_req.auth_json,
            disk_type: create_req.disk_type,
            labels: vec![],
            created_at: ic_cdk::api::time() / 1_000_000,
            root_folder: root_folder_uuid,
            trash_folder: trash_folder_uuid,
            external_id: new_external_id.clone(),
            external_payload: Some(ExternalPayload(create_req.external_payload.unwrap_or("".to_string()))),
            endpoint: create_req.endpoint,
        };
        update_external_id_mapping(
            None,
            new_external_id,
            Some(disk_id.0.clone())
        );

        // Store the disk
        DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(disk_id.clone(), disk.clone());
        });

        DISKS_BY_TIME_LIST.with(|store| {
            store.borrow_mut().push(&disk_id.clone());
        });
        mark_claimed_uuid(&disk_id.clone().to_string());


        snapshot_poststate(prestate, Some(
            format!(
                "{}: Create Disk {}",
                requester_api_key.user_id,
                disk_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            CreateDiskResponse::ok(&disk.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn update_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());


        // Parse request body
        let body: &[u8] = request.body();
        let update_req = serde_json::from_slice::<UpdateDiskRequestBody>(body).unwrap();

        if let Err(validation_error) = update_req.validate_body() {
            return create_response(
                StatusCode::BAD_REQUEST,
                ErrorResponse::err(400, validation_error.message).encode()
            );
        }

        let disk_id = DiskID(update_req.id);

        // Get existing disk
        let mut disk = match DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().get(&disk_id).map(|d| d.clone())) {
            Some(disk) => disk,
            None => return create_response(
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found().encode()
            ),
        };

        // Check update permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Disks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            if !permissions.contains(&SystemPermissionType::Edit) && !table_permissions.contains(&SystemPermissionType::Edit) {
                return create_auth_error_response();
            }
        }
        let prestate = snapshot_prestate();

        // Update fields
        if let Some(private_note) = update_req.private_note {
            disk.private_note = Some(private_note);
        }
        if let Some(auth_json) = update_req.auth_json {
            // Validate auth_json if provided
            if let Err(err_msg) = validate_auth_json(&disk.disk_type, &Some(auth_json.clone())) {
                return create_response(
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::err(400, err_msg).encode()
                );
            }
            disk.auth_json = Some(auth_json);
        }
        if let Some(name) = update_req.name {
            disk.name = name;
        }
        if let Some(public_note) = update_req.public_note {
            disk.public_note = Some(public_note);
        }
        if let Some(external_id) = update_req.external_id {
            let old_external_id = disk.external_id.clone();
            let new_external_id = Some(ExternalID(external_id));
            disk.external_id = new_external_id.clone();
            update_external_id_mapping(
                old_external_id,
                new_external_id,
                Some(disk_id.0.clone())
            );
        }
        if let Some(external_payload) = update_req.external_payload {
            disk.external_payload = Some(ExternalPayload(external_payload));
        }
        if let Some(endpoint) = update_req.endpoint {
            disk.endpoint = Some(endpoint);
        }

        DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().insert(disk_id.clone(), disk.clone());
        });

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Update Disk {}",
                requester_api_key.user_id,
                disk_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            UpdateDiskResponse::ok(&disk.cast_fe(&requester_api_key.user_id)).encode()
        )
    }

    pub async fn delete_disk_handler<'a, 'k, 'v>(request: &'a HttpRequest<'a>, params: &'a Params<'k, 'v>) -> HttpResponse<'static> {
        // Authenticate request
        let requester_api_key = match authenticate_request(request) {
            Some(key) => key,
            None => return create_auth_error_response(),
        };

        let is_owner = OWNER_ID.with(|owner_id| requester_api_key.user_id == *owner_id.borrow().get());

        let prestate = snapshot_prestate();

        // Parse request body
        let body: &[u8] = request.body();
        let delete_request = match serde_json::from_slice::<DeleteDiskRequest>(body) {
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

        let disk_id = delete_request.id.clone();

        // Check delete permission if not owner
        if !is_owner {
            let table_permissions = check_system_permissions(
                SystemResourceID::Table(SystemTableEnum::Disks),
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );
            let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.to_string()));
            let permissions = check_system_permissions(
                resource_id,
                PermissionGranteeID::User(requester_api_key.user_id.clone())
            );

            if !permissions.contains(&SystemPermissionType::Delete) || !table_permissions.contains(&SystemPermissionType::Delete) {
                return create_auth_error_response();
            }
        }

        // Get disk for external ID cleanup
        let disk = DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow().get(&disk_id).map(|d| d.clone())
        });

        // Remove from main stores
        DISKS_BY_ID_HASHTABLE.with(|store| {
            store.borrow_mut().remove(&disk_id);
        });

        // For removing items from DISKS_BY_TIME_LIST
        DISKS_BY_TIME_LIST.with(|store| {
            let mut new_vec = StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DISKS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize new StableVec");

            // Copy all items except the one to be deleted
            let store_ref = store.borrow();
            for i in 0..store_ref.len() {
                if let Some(id) = store_ref.get(i) {
                    if id != disk_id {
                        new_vec.push(&id);
                    }
                }
            }

            // Replace the old vector with the new one
            drop(store_ref);
            *store.borrow_mut() = new_vec;
        });

        // Remove from external ID mappings
        if let Some(disk) = disk {
            update_external_id_mapping(
                disk.external_id,
                None,
                Some(disk.id.to_string()),
            );
        }

        snapshot_poststate(prestate, Some(
            format!(
                "{}: Delete Disk {}",
                requester_api_key.user_id,
                disk_id.clone()
            ).to_string())
        );

        create_response(
            StatusCode::OK,
            DeleteDiskResponse::ok(&DeletedDiskData {
                id: disk_id,
                deleted: true
            }).encode()
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
