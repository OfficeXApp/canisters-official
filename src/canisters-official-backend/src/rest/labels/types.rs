// src/rest/labels/types.rs

use serde::{Deserialize, Serialize};
use crate::core::api::permissions::system::check_system_permissions;
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum};
use crate::core::state::labels::state::validate_uuid4_string_with_prefix;
use crate::core::state::labels::types::{redact_label, Label, LabelID, LabelResourceID};
use crate::core::types::{ClientSuggestedUUID, IDPrefix, UserID};
use crate::rest::webhooks::types::SortDirection;
use crate::rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_short_string, validate_unclaimed_uuid, ApiResponse, UpsertActionTypeEnum, ValidationError};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelFE {
    #[serde(flatten)] 
    pub label: Label,
    pub permission_previews: Vec<SystemPermissionType>, 
}

impl LabelFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // we redact the label value for non-owners as it may leak sensitive info about the organization
            redacted.label.resources = vec![];

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.label.private_note = None;
            }
        }
        // Filter labels
        redacted.label.labels = match is_owner {
            true => redacted.label.labels,
            false => redacted.label.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListLabelsRequestBody {
    #[serde(default)]
    pub filters: ListLabelsRequestBodyFilters,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

fn default_page_size() -> usize {
    50
}

impl ListLabelsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate page_size is reasonable
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(ValidationError {
                field: "page_size".to_string(),
                message: "Page size must be between 1 and 1000".to_string(),
            });
        }

        // Validate prefix if present
        if let Some(prefix) = &self.filters.prefix {
            if prefix.len() > 256 {
                return Err(ValidationError {
                    field: "filters.prefix".to_string(),
                    message: "Prefix filter must be 256 characters or less".to_string(),
                });
            }
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


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListLabelsRequestBodyFilters {
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListLabelsResponseData {
    pub items: Vec<LabelFE>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateLabelRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub value: String,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub color: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateLabelRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::LabelID)?;
        }
        
        // Validate label value (up to 256 chars)
        validate_short_string(&self.value, "value")?;

        // Validate description if provided
        if let Some(public_note) = &self.public_note {
            validate_description(public_note, "public_note")?;
        }
        if let Some(private_note) = &self.private_note {
            validate_description(private_note, "private_note")?;
        }

        // Validate color if provided
        if let Some(color) = &self.color {
            // Basic hex color validation
            if !color.starts_with('#') || (color.len() != 7 && color.len() != 4) {
                return Err(ValidationError {
                    field: "color".to_string(),
                    message: "Color must be a valid hex color code (e.g., #RRGGBB or #RGB)".to_string(),
                });
            }
            
            // Verify all characters after # are valid hex
            if !color[1..].chars().all(|c| c.is_digit(16)) {
                return Err(ValidationError {
                    field: "color".to_string(),
                    message: "Color must contain only valid hexadecimal characters".to_string(),
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
pub struct UpdateLabelRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}
impl UpdateLabelRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate label ID
        validate_id_string(&self.id, "id")?;

        // Validate label value if provided
        if let Some(value) = &self.value {
            validate_short_string(value, "value")?;
        }

        // Validate description if provided
        if let Some(public_note) = &self.public_note {
            validate_description(public_note, "public_note")?;
        }
        if let Some(private_note) = &self.private_note {
            validate_description(private_note, "private_note")?;
        }

        // Validate color if provided
        if let Some(color) = &self.color {
            // Basic hex color validation
            if !color.starts_with('#') || (color.len() != 7 && color.len() != 4) {
                return Err(ValidationError {
                    field: "color".to_string(),
                    message: "Color must be a valid hex color code (e.g., #RRGGBB or #RGB)".to_string(),
                });
            }
            
            // Verify all characters after # are valid hex
            if !color[1..].chars().all(|c| c.is_digit(16)) {
                return Err(ValidationError {
                    field: "color".to_string(),
                    message: "Color must contain only valid hexadecimal characters".to_string(),
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
pub struct DeleteLabelRequest {
    pub id: String,
}
impl DeleteLabelRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate label ID
        validate_id_string(&self.id, "id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedLabelData {
    pub id: LabelID,
    pub deleted: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LabelResourceRequest {
    pub label_id: String,
    pub resource_id: String,
    pub add: bool,  // true to add, false to remove
}
impl LabelResourceRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate label ID
        validate_id_string(&self.label_id, "label_id")?;
        
        // Validate resource ID
        validate_id_string(&self.resource_id, "resource_id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelOperationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub label: Option<LabelFE>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetLabelResourcesRequest {
    pub label_id: String,
    pub resource_type: Option<String>,
    pub page_size: Option<usize>,
    pub cursor: Option<String>,
}
impl GetLabelResourcesRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate label ID
        validate_id_string(&self.label_id, "label_id")?;
        
        // Validate resource_type if provided
        if let Some(resource_type) = &self.resource_type {
            validate_short_string(resource_type, "resource_type")?;
        }
        
        // Validate page_size if provided
        if let Some(page_size) = self.page_size {
            if page_size == 0 || page_size > 1000 {
                return Err(ValidationError {
                    field: "page_size".to_string(),
                    message: "Page size must be between 1 and 1000".to_string(),
                });
            }
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
pub struct GetLabelResourcesResponseData {
    pub label_id: String,
    pub resources: Vec<LabelResourceID>,
    pub page_size: usize,
    pub total: usize,
    pub cursor: Option<String>,
}

pub type GetLabelResponse<'a> = ApiResponse<'a, LabelFE>;
pub type DeleteLabelResponse<'a> = ApiResponse<'a, DeletedLabelData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ListLabelsResponse<'a> = ApiResponse<'a, ListLabelsResponseData>;
pub type CreateLabelResponse<'a> = ApiResponse<'a, LabelFE>;
pub type UpdateLabelResponse<'a> = ApiResponse<'a, LabelFE>;
pub type LabelResourceResponse<'a> = ApiResponse<'a, LabelOperationResponse>;
pub type GetLabelResourcesResponse<'a> = ApiResponse<'a, GetLabelResourcesResponseData>;
