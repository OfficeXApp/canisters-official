// src/rest/drives/types.rs

use serde::{Deserialize, Serialize};
use crate::core::api::permissions::system::check_system_permissions;
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::drives::types::{Drive, DriveID, DriveStateDiffID, ExternalID, StateChecksum, StateDiffRecord};
use crate::core::state::permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum};
use crate::core::state::search::types::{SearchCategoryEnum, SearchResult};
use crate::core::state::tags::state::validate_uuid4_string_with_prefix;
use crate::core::state::tags::types::redact_tag;
use crate::core::types::{ClientSuggestedUUID, ICPPrincipalString, IDPrefix, PublicKeyICP, UserID};
use crate::rest::webhooks::types::{SortDirection};
use crate::rest::types::{validate_drive_id, validate_external_id, validate_external_payload, validate_icp_principal, validate_id_string, ApiResponse, UpsertActionTypeEnum, ValidationError};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFE {
    #[serde(flatten)] 
    pub drive: Drive,
    pub permission_previews: Vec<SystemPermissionType>, 
}

impl DriveFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.drive.private_note = None;
            }
        }
        // Filter tags
        redacted.drive.tags = match is_owner {
            true => redacted.drive.tags,
            false => redacted.drive.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}




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
    pub items: Vec<DriveFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDriveRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub name: String,
    pub icp_principal: String,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateDriveRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_drive_id(&self.id.as_ref().unwrap().to_string())?;
        }
        
        // Validate name (up to 256 chars)
        validate_id_string(&self.name, "name")?;

        // Validate ICP principal if provided
        validate_icp_principal(&self.icp_principal)?;

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

        // Validate endpoint_url if provided
        if let Some(endpoint_url) = &self.endpoint_url {
            if endpoint_url.len() > 4096 {
                return Err(ValidationError {
                    field: "endpoint_url".to_string(),
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
    pub endpoint_url: Option<String>,
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

        // Validate endpoint_url if provided
        if let Some(endpoint_url) = &self.endpoint_url {
            if endpoint_url.len() > 4096 {
                return Err(ValidationError {
                    field: "endpoint_url".to_string(),
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

pub type GetDriveResponse<'a> = ApiResponse<'a, DriveFE>;
pub type ListDrivesResponse<'a> = ApiResponse<'a, ListDrivesResponseData>;
pub type CreateDriveResponse<'a> = ApiResponse<'a, DriveFE>;
pub type UpdateDriveResponse<'a> = ApiResponse<'a, DriveFE>;

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


