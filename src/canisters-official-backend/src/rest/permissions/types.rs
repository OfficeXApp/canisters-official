// src/rest/permissions/types.rs
use serde::{Deserialize, Serialize};
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::permissions::types::*;
use crate::core::state::tags::types::redact_tag;
use crate::core::types::UserID;
use crate::rest::directory::types::DirectoryResourceID;
use crate::core::state::permissions::types::PermissionMetadata;
use crate::rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, ApiResponse, ValidationError};






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPermissionFE {
    #[serde(flatten)] 
    pub system_permission: SystemPermission,
    pub permission_previews: Vec<SystemPermissionType>, 
}

impl SystemPermissionFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                // redacted.system_permission.private_note = None;
            }
        }
        // Filter tags
        redacted.system_permission.tags = match is_owner {
            true => redacted.system_permission.tags,
            false => redacted.system_permission.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryPermissionFE {
    #[serde(flatten)] 
    pub directory_permission: DirectoryPermission,
    pub permission_previews: Vec<SystemPermissionType>, 
}

impl DirectoryPermissionFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                // redacted.system_permission.private_note = None;
            }
        }
        // Filter tags
        redacted.directory_permission.tags = match is_owner {
            true => redacted.directory_permission.tags,
            false => redacted.directory_permission.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}



// Response type included in FileRecord/FolderRecord
#[derive(Debug, Clone, Serialize)]
pub struct ResourcePermissionInfo {
    pub user_permissions: Vec<DirectoryPermissionType>,
    pub is_owner: bool,
    pub can_share: bool,
}

// Create Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDirectoryPermissionsRequestBody {
    pub resource_id: String,
    pub granted_to: Option<String>,
    pub permission_types: Vec<DirectoryPermissionType>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub inheritable: Option<bool>,
    pub note: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl CreateDirectoryPermissionsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate resource_id
        validate_id_string(&self.resource_id, "resource_id")?;
        
        // Validate granted_to if provided
        if let Some(granted_to) = &self.granted_to {
            validate_id_string(granted_to, "granted_to")?;
        }
        
        // Validate permission_types (must not be empty)
        if self.permission_types.is_empty() {
            return Err(ValidationError {
                field: "permission_types".to_string(),
                message: "Permission types cannot be empty".to_string(),
            });
        }
        
        // Validate note if provided
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
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


#[derive(Debug, Clone, Serialize)]
pub struct CreateDirectoryPermissionsResponseData {
    pub permission: DirectoryPermissionFE,
}


// Update Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDirectoryPermissionsRequestBody {
    pub id: DirectoryPermissionID,
    pub resource_id: String,
    pub granted_to: Option<String>,
    pub permission_types: Vec<DirectoryPermissionType>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub inheritable: Option<bool>,
    pub note: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl UpdateDirectoryPermissionsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        validate_id_string(&self.id.0, "id")?;

        // Validate resource_id
        validate_id_string(&self.resource_id, "resource_id")?;
        
        // Validate granted_to if provided
        if let Some(granted_to) = &self.granted_to {
            validate_id_string(granted_to, "granted_to")?;
        }
        
        // Validate permission_types (must not be empty)
        if self.permission_types.is_empty() {
            return Err(ValidationError {
                field: "permission_types".to_string(),
                message: "Permission types cannot be empty".to_string(),
            });
        }
        
        // Validate note if provided
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
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


#[derive(Debug, Clone, Serialize)]
pub struct UpdateDirectoryPermissionsResponseData {
    pub permission: DirectoryPermissionFE,
}


// Delete Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct DeletePermissionRequest {
    pub permission_id: DirectoryPermissionID,
}

impl DeletePermissionRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate permission_id
        validate_id_string(&self.permission_id.0, "permission_id")?;
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DeletePermissionResponseData {
    pub deleted_id: DirectoryPermissionID,
}

// Check Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct PermissionCheckRequest {
    pub resource_id: String,
    pub grantee_id: String,
}

impl PermissionCheckRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate resource_id
        validate_id_string(&self.resource_id, "resource_id")?;
        
        // Validate grantee_id
        validate_id_string(&self.grantee_id, "grantee_id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckPermissionResult {
    pub resource_id: DirectoryResourceID,
    pub grantee_id: PermissionGranteeID,
    pub permissions: Vec<DirectoryPermissionType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedeemPermissionRequest {
    pub permission_id: String,
    pub user_id: String,
}

impl RedeemPermissionRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate permission_id
        validate_id_string(&self.permission_id, "permission_id")?;
        
        // Validate user_id
        validate_id_string(&self.user_id, "user_id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedeemPermissionResponseData {
    pub permission: DirectoryPermissionFE,
}

pub type RedeemPermissionResponse<'a> = ApiResponse<'a, RedeemPermissionResponseData>;


// Response type aliases using ApiResponse
pub type GetPermissionResponse<'a> = ApiResponse<'a, DirectoryPermissionFE>;
pub type CreatePermissionsResponse<'a> = ApiResponse<'a, CreateDirectoryPermissionsResponseData>;
pub type DeletePermissionResponse<'a> = ApiResponse<'a, DeletePermissionResponseData>;
pub type CheckPermissionResponse<'a> = ApiResponse<'a, CheckPermissionResult>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;



// Get System Permission
#[derive(Debug, Clone, Deserialize)] 
pub struct GetSystemPermissionRequest {
    pub permission_id: SystemPermissionID,
}
impl GetSystemPermissionRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate permission_id
        validate_id_string(&self.permission_id.0, "permission_id")?;
        
        Ok(())
    }
}

// Create System Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSystemPermissionsRequestBody {
    pub resource_id: String, // Can be "Table_drives" or "DiskID_123" etc
    pub granted_to: Option<String>,
    pub permission_types: Vec<SystemPermissionType>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub note: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateSystemPermissionsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate resource_id
        validate_id_string(&self.resource_id, "resource_id")?;
        
        // Validate granted_to if provided
        if let Some(granted_to) = &self.granted_to {
            validate_id_string(granted_to, "granted_to")?;
        }
        
        // Validate permission_types (must not be empty)
        if self.permission_types.is_empty() {
            return Err(ValidationError {
                field: "permission_types".to_string(),
                message: "Permission types cannot be empty".to_string(),
            });
        }
        
        // Validate note if provided
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
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

#[derive(Debug, Clone, Serialize)]
pub struct CreateSystemPermissionsResponseData {
    pub permission: SystemPermissionFE,
}


// Update System Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSystemPermissionsRequestBody {
    pub id: SystemPermissionID,
    pub resource_id: String, // Can be "Table_drives" or "DiskID_123" etc
    pub granted_to: Option<String>,
    pub permission_types: Vec<SystemPermissionType>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub note: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl UpdateSystemPermissionsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        validate_id_string(&self.id.0, "id")?;

        // Validate resource_id
        validate_id_string(&self.resource_id, "resource_id")?;
        
        // Validate granted_to if provided
        if let Some(granted_to) = &self.granted_to {
            validate_id_string(granted_to, "granted_to")?;
        }
        
        // Validate permission_types (must not be empty)
        if self.permission_types.is_empty() {
            return Err(ValidationError {
                field: "permission_types".to_string(),
                message: "Permission types cannot be empty".to_string(),
            });
        }
        
        // Validate note if provided
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
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

#[derive(Debug, Clone, Serialize)]
pub struct UpdateSystemPermissionsResponseData {
    pub permission: SystemPermissionFE,
}


// Delete System Permission
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteSystemPermissionRequest {
    pub permission_id: SystemPermissionID,
}
impl DeleteSystemPermissionRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate permission_id
        validate_id_string(&self.permission_id.0, "permission_id")?;
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DeleteSystemPermissionResponseData {
    pub deleted_id: SystemPermissionID,
}

// Check System Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct SystemPermissionCheckRequest {
    pub resource_id: String,
    pub grantee_id: String,
}
impl SystemPermissionCheckRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate resource_id
        validate_id_string(&self.resource_id, "resource_id")?;
        
        // Validate grantee_id
        validate_id_string(&self.grantee_id, "grantee_id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckSystemPermissionResult {
    pub resource_id: SystemResourceID,
    pub grantee_id: PermissionGranteeID,
    pub permissions: Vec<SystemPermissionType>,
}

// Redeem System Permission
#[derive(Debug, Clone, Deserialize)]
pub struct RedeemSystemPermissionRequest {
    pub permission_id: String,
    pub user_id: String,
}
impl RedeemSystemPermissionRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate permission_id
        validate_id_string(&self.permission_id, "permission_id")?;
        
        // Validate user_id
        validate_id_string(&self.user_id, "user_id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedeemSystemPermissionResponseData {
    pub permission: SystemPermissionFE,
}

// Response type aliases
pub type GetSystemPermissionResponse<'a> = ApiResponse<'a, SystemPermissionFE>;
pub type CreateSystemPermissionsResponse<'a> = ApiResponse<'a, CreateSystemPermissionsResponseData>;
pub type DeleteSystemPermissionResponse<'a> = ApiResponse<'a, DeleteSystemPermissionResponseData>;
pub type CheckSystemPermissionResponse<'a> = ApiResponse<'a, CheckSystemPermissionResult>;
pub type RedeemSystemPermissionResponse<'a> = ApiResponse<'a, RedeemSystemPermissionResponseData>;
