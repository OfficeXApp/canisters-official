// src/rest/organization/types.rs

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use crate::core::state::drives::types::{Drive, DriveID, DriveStateDiffID, ExternalID, StateChecksum, StateDiffRecord};
use crate::core::state::search::types::{SearchCategoryEnum, SearchResult};
use crate::core::types::{ICPPrincipalString, PublicKeyICP, UserID};
use crate::rest::webhooks::types::{SortDirection};
use crate::rest::types::{validate_drive_id, validate_external_id, validate_external_payload, validate_icp_principal, validate_id_string, validate_seed_phrase, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError};

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


fn default_page_size() -> usize {
    50
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



#[derive(Debug, Clone, Serialize)]
pub struct WhoAmIReport {
    pub nickname: String,
    pub userID: UserID,
    pub driveID: DriveID,
    pub icp_principal: ICPPrincipalString,
    pub evm_public_address: Option<String>,
    pub is_owner: bool,
    pub drive_nickname: String,
}
pub type GetWhoAmIResponse<'a> = ApiResponse<'a, WhoAmIReport>;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperswapUserIDRequestBody {
    pub current_user_id: String,
    pub new_user_id: String,
}
impl SuperswapUserIDRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate that they are different and users
        if self.current_user_id == self.new_user_id {
            return Err(ValidationError {
                field: "new_user_id".to_string(),
                message: "New user ID must be different from current user ID".to_string(),
            });
        }

        // Validate current_user_id format with validate_user_id
        validate_user_id(&self.current_user_id)?;
        validate_user_id(&self.new_user_id)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperswapUserIDResponseData {
    pub success: bool,
    pub message: String,
}
pub type SuperswapUserIDResponse<'a> = ApiResponse<'a, SuperswapUserIDResponseData>;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemOrgRequestBody {
    pub redeem_code: String,
}
impl RedeemOrgRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        
        // validate the redeem_code is a valid redeem code
        validate_id_string(&self.redeem_code, "redeem_code")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemOrgResponseData {
    pub drive_id: DriveID, // spawned drive id
    pub endpoint_url: String, // spawned drive url endpoint
    pub api_key: String, // admin api key for the spawned drive
    pub note: String, // note about the spawned drive, particularly info about the factory
    pub admin_login_password: String, // admin login password for the spawned drive
}
pub type RedeemOrgResponse<'a> = ApiResponse<'a, RedeemOrgResponseData>;

