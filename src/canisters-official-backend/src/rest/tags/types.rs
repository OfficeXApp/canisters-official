// src/rest/tags/types.rs

use serde::{Deserialize, Serialize};
use crate::core::api::permissions::system::check_system_permissions;
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum};
use crate::core::state::tags::state::validate_uuid4_string_with_prefix;
use crate::core::state::tags::types::{redact_tag, Tag, TagID, TagResourceID};
use crate::core::types::{ClientSuggestedUUID, IDPrefix, UserID};
use crate::rest::webhooks::types::SortDirection;
use crate::rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, ApiResponse, UpsertActionTypeEnum, ValidationError};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFE {
    #[serde(flatten)] 
    pub tag: Tag,
    pub permission_previews: Vec<SystemPermissionType>, 
}

impl TagFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // we redact the tag value for non-owners as it may leak sensitive info about the organization
            redacted.tag.resources = vec![];

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.tag.private_note = None;
            }
        }
        // Filter tags
        redacted.tag.tags = match is_owner {
            true => redacted.tag.tags,
            false => redacted.tag.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTagsRequestBody {
    #[serde(default)]
    pub filters: ListTagsRequestBodyFilters,
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

impl ListTagsRequestBody {
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


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTagsRequestBodyFilters {
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListTagsResponseData {
    pub items: Vec<TagFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTagRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub value: String,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub color: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateTagRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::TagID)?;
        }
        
        // Validate tag value (up to 256 chars)
        validate_id_string(&self.value, "value")?;

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
pub struct UpdateTagRequestBody {
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
impl UpdateTagRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate tag ID
        validate_id_string(&self.id, "id")?;

        // Validate tag value if provided
        if let Some(value) = &self.value {
            validate_id_string(value, "value")?;
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
pub struct DeleteTagRequest {
    pub id: String,
}
impl DeleteTagRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate tag ID
        validate_id_string(&self.id, "id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedTagData {
    pub id: TagID,
    pub deleted: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagResourceRequest {
    pub tag_id: String,
    pub resource_id: String,
    pub add: bool,  // true to add, false to remove
}
impl TagResourceRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate tag ID
        validate_id_string(&self.tag_id, "tag_id")?;
        
        // Validate resource ID
        validate_id_string(&self.resource_id, "resource_id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TagOperationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub tag: Option<TagFE>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTagResourcesRequest {
    pub tag_id: String,
    pub resource_type: Option<String>,
    pub page_size: Option<usize>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}
impl GetTagResourcesRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate tag ID
        validate_id_string(&self.tag_id, "tag_id")?;
        
        // Validate resource_type if provided
        if let Some(resource_type) = &self.resource_type {
            validate_id_string(resource_type, "resource_type")?;
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
pub struct GetTagResourcesResponseData {
    pub tag_id: String,
    pub resources: Vec<TagResourceID>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub type GetTagResponse<'a> = ApiResponse<'a, TagFE>;
pub type DeleteTagResponse<'a> = ApiResponse<'a, DeletedTagData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ListTagsResponse<'a> = ApiResponse<'a, ListTagsResponseData>;
pub type CreateTagResponse<'a> = ApiResponse<'a, TagFE>;
pub type UpdateTagResponse<'a> = ApiResponse<'a, TagFE>;
pub type TagResourceResponse<'a> = ApiResponse<'a, TagOperationResponse>;
pub type GetTagResourcesResponse<'a> = ApiResponse<'a, GetTagResourcesResponseData>;
