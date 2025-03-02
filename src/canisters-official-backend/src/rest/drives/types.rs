// src/rest/drives/types.rs

use serde::{Deserialize, Serialize};
use crate::core::state::drives::types::{Drive, DriveID, DriveStateDiffID, ExternalID, StateChecksum, StateDiffRecord};
use crate::core::state::search::types::{SearchCategoryEnum, SearchResult};
use crate::core::types::PublicKeyICP;
use crate::rest::webhooks::types::{SortDirection};
use crate::rest::types::{validate_drive_id, validate_external_id, validate_external_payload, validate_icp_principal, validate_id_string, ApiResponse, ValidationError};

#[derive(Debug, Clone, Deserialize)]
pub struct ListDrivesRequestBody {
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

fn default_page_size() -> usize {
    50
}

impl ListDrivesRequestBody {
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
        if let Some(cursor) = &self.cursor_up {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_up".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        if let Some(cursor) = &self.cursor_down {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_down".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct ListDrivesResponseData {
    pub items: Vec<Drive>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertDriveRequestBody {
    Create(CreateDriveRequestBody),
    Update(UpdateDriveRequestBody),
}

impl UpsertDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertDriveRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertDriveRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDriveRequestBody {
    pub name: String,
    pub icp_principal: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub url_endpoint: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate name (up to 256 chars)
        validate_id_string(&self.name, "name")?;

        // Validate ICP principal if provided
        if let Some(icp_principal) = &self.icp_principal {
            validate_icp_principal(icp_principal)?;
        }

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

        // Validate url_endpoint if provided
        if let Some(url_endpoint) = &self.url_endpoint {
            if url_endpoint.len() > 4096 {
                return Err(ValidationError {
                    field: "url_endpoint".to_string(),
                    message: "URL endpoint must be 4,096 characters or less".to_string(),
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
pub struct UpdateDriveRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icp_principal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}
impl UpdateDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate the DriveID
        validate_drive_id(&self.id)?;

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_id_string(name, "name")?;
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

        // Validate ICP principal if provided
        if let Some(icp_principal) = &self.icp_principal {
            validate_icp_principal(icp_principal)?;
        }

        // Validate url_endpoint if provided
        if let Some(url_endpoint) = &self.url_endpoint {
            if url_endpoint.len() > 4096 {
                return Err(ValidationError {
                    field: "url_endpoint".to_string(),
                    message: "URL endpoint must be 4,096 characters or less".to_string(),
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

pub type GetDriveResponse<'a> = ApiResponse<'a, Drive>;
pub type ListDrivesResponse<'a> = ApiResponse<'a, ListDrivesResponseData>;
pub type CreateDriveResponse<'a> = ApiResponse<'a, Drive>;
pub type UpdateDriveResponse<'a> = ApiResponse<'a, Drive>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteDriveRequest {
    pub id: DriveID,
}
impl DeleteDriveRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate the DriveID
        validate_drive_id(&self.id.0)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedDriveData {
    pub id: DriveID,
    pub deleted: bool
}

pub type DeleteDriveResponse<'a> = ApiResponse<'a, DeletedDriveData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDriveRequestBody {
    pub diffs: Vec<StateDiffRecord>,
    pub notes: Option<String>,
}
impl ReplayDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate that diffs are provided
        if self.diffs.is_empty() {
            return Err(ValidationError {
                field: "diffs".to_string(),
                message: "No diffs provided for replay".to_string(),
            });
        }

        // Validate notes if provided
        if let Some(notes) = &self.notes {
            if notes.len() > 8192 {
                return Err(ValidationError {
                    field: "notes".to_string(),
                    message: "Notes must be 8,192 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDriveResponseData {
    pub timestamp_ns: u64,
    pub diffs_applied: usize,
    pub checkpoint_diff_id: Option<DriveStateDiffID>,
    pub final_checksum: StateChecksum,
}

pub type ReplayDriveResponse<'a> = ApiResponse<'a, ReplayDriveResponseData>;



#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchSortByEnum {
    #[serde(rename = "CREATED_AT")]
    CreatedAt,
    #[serde(rename = "UPDATED_AT")]
    UpdatedAt,
}

impl Default for SearchSortByEnum {
    fn default() -> Self {
        SearchSortByEnum::UpdatedAt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDriveRequestBody {
    pub query: String,
    #[serde(default)]
    pub categories: Vec<SearchCategoryEnum>,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
    #[serde(default)]
    pub sort_by: SearchSortByEnum,
    #[serde(default)]
    pub direction: SortDirection,
}
impl SearchDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate query (cannot be empty and should have a reasonable length)
        if self.query.trim().is_empty() {
            return Err(ValidationError {
                field: "query".to_string(),
                message: "Search query cannot be empty".to_string(),
            });
        }

        if self.query.len() > 256 {
            return Err(ValidationError {
                field: "query".to_string(),
                message: "Search query must be 256 characters or less".to_string(),
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
        if let Some(cursor) = &self.cursor_up {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_up".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        if let Some(cursor) = &self.cursor_down {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_down".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDriveResponseData {
    pub items: Vec<SearchResult>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub type SearchDriveResponse<'a> = ApiResponse<'a, SearchDriveResponseData>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexDriveRequestBody {
    // Optional field to override the 5 minute rate limit
    pub force: Option<bool>,
}
impl ReindexDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // No specific validation needed for this simple structure
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReindexDriveResponseData {
    pub success: bool,
    pub timestamp_ms: u64,
    pub indexed_count: usize,
}

pub type ReindexDriveResponse<'a> = ApiResponse<'a, ReindexDriveResponseData>;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIDsDriveRequestBody {
    pub external_ids: Vec<ExternalID>,
}
impl ExternalIDsDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // No validation needed if the list is empty - that's a valid case
        
        // Validate each external ID in the list
        for (index, external_id) in self.external_ids.iter().enumerate() {
            if external_id.0.len() > 256 {
                return Err(ValidationError {
                    field: format!("external_ids[{}]", index),
                    message: "External ID must be 256 characters or less".to_string(),
                });
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIDsDriveResponseData {
    pub results: Vec<ExternalIDvsInternalIDMaps>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIDvsInternalIDMaps {
    pub success: bool,
    pub message: String,
    pub external_id: ExternalID,
    pub internal_ids: Vec<String>,
}

pub type ExternalIDsDriveResponse<'a> = ApiResponse<'a, ExternalIDsDriveResponseData>;

#[derive(Debug, Clone, Deserialize)]
pub struct TransferOwnershipDriveRequestBody {
    pub next_owner_id: String,
}
impl TransferOwnershipDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate next_owner_id format
        validate_id_string(&self.next_owner_id, "next_owner_id")?;
        
        // Check if next_owner_id starts with the correct prefix
        let user_prefix = crate::core::types::IDPrefix::User.as_str();
        if !self.next_owner_id.starts_with(user_prefix) {
            return Err(ValidationError {
                field: "next_owner_id".to_string(),
                message: format!("Next owner ID must start with '{}'", user_prefix),
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferOwnershipStatusEnum {
    Requested,
    Completed,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferOwnershipResponseData {
    pub status: TransferOwnershipStatusEnum,
    pub ready_ms: u64,
}

pub type TransferOwnershipDriveResponse<'a> = ApiResponse<'a, TransferOwnershipResponseData>;