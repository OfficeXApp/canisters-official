// src/rest/disks/types.rs

use serde::{Deserialize, Serialize};

use crate::{
    core::state::disks::types::{Disk, DiskID, DiskTypeEnum},
    rest::{types::{validate_external_id, validate_external_payload, validate_id_string, ApiResponse, ValidationError}, webhooks::types::SortDirection},
};

#[derive(Debug, Clone, Deserialize)]
pub struct ListDisksRequestBody {
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
pub struct ListDisksResponseData {
    pub items: Vec<Disk>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertDiskRequestBody {
    Create(CreateDiskRequestBody),
    Update(UpdateDiskRequestBody),
}

impl UpsertDiskRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertDiskRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertDiskRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDiskRequestBody {
    pub name: String,
    pub disk_type: DiskTypeEnum,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub auth_json: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateDiskRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate name (up to 256 chars)
        validate_id_string(&self.name, "name")?;

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

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDiskRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}
impl UpdateDiskRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate ID string
        validate_id_string(&self.id, "id")?;

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

pub type GetDiskResponse<'a> = ApiResponse<'a, Disk>;
pub type DeleteDiskResponse<'a> = ApiResponse<'a, DeletedDiskData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ListDisksResponse<'a> = ApiResponse<'a, ListDisksResponseData>;
pub type CreateDiskResponse<'a> = ApiResponse<'a, Disk>;
pub type UpdateDiskResponse<'a> = ApiResponse<'a, Disk>;
