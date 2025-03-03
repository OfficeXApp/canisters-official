// src/rest/permissions/types.rs
use serde::{Deserialize, Serialize};
use crate::core::state::permissions::types::*;
use crate::rest::directory::types::DirectoryResourceID;
use crate::core::state::permissions::types::PermissionMetadata;
use crate::rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, ApiResponse, UpsertActionTypeEnum, ValidationError};

// Response type included in FileRecord/FolderRecord
#[derive(Debug, Clone, Serialize)]
pub struct ResourcePermissionInfo {
    pub user_permissions: Vec<DirectoryPermissionType>,
    pub is_owner: bool,
    pub can_share: bool,
}

// Upsert Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct UpsertPermissionsRequestBody {
    pub action: UpsertActionTypeEnum,
    pub id: Option<DirectoryPermissionID>,
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

impl UpsertPermissionsRequestBody {
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
        
        // Validate ID if provided
        if let Some(id) = &self.id {
            validate_id_string(&id.0, "id")?;
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct UpsertPermissionsResponseData {
    pub permission: DirectoryPermission,
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
    pub permission: DirectoryPermission,
}

pub type RedeemPermissionResponse<'a> = ApiResponse<'a, RedeemPermissionResponseData>;


// Response type aliases using ApiResponse
pub type GetPermissionResponse<'a> = ApiResponse<'a, DirectoryPermission>;
pub type UpsertPermissionsResponse<'a> = ApiResponse<'a, UpsertPermissionsResponseData>;
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

// Upsert System Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct UpsertSystemPermissionsRequestBody {
    pub id: Option<SystemPermissionID>,
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
impl UpsertSystemPermissionsRequestBody {
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
        
        // Validate ID if provided
        if let Some(id) = &self.id {
            validate_id_string(&id.0, "id")?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpsertSystemPermissionsResponseData {
    pub permission: SystemPermission,
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
    pub permission: SystemPermission,
}

// Response type aliases
pub type GetSystemPermissionResponse<'a> = ApiResponse<'a, SystemPermission>;
pub type UpsertSystemPermissionsResponse<'a> = ApiResponse<'a, UpsertSystemPermissionsResponseData>;
pub type DeleteSystemPermissionResponse<'a> = ApiResponse<'a, DeleteSystemPermissionResponseData>;
pub type CheckSystemPermissionResponse<'a> = ApiResponse<'a, CheckSystemPermissionResult>;
pub type RedeemSystemPermissionResponse<'a> = ApiResponse<'a, RedeemSystemPermissionResponseData>;
