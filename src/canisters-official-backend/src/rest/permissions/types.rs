// src/rest/permissions/types.rs
use serde::{Deserialize, Serialize};
use crate::core::state::directory::types::{DriveClippedFilePath, DriveFullFilePath};
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::drives::types::{ExternalID, ExternalPayload};
use crate::core::state::permissions::types::*;
use crate::core::state::labels::state::validate_uuid4_string_with_prefix;
use crate::core::state::labels::types::{redact_label, LabelStringValue};
use crate::core::types::{ClientSuggestedUUID, IDPrefix, UserID};
use crate::rest::directory::types::DirectoryResourceID;
use crate::core::state::permissions::types::PermissionMetadata;
use crate::rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_unclaimed_uuid, ApiResponse, ValidationError};
use crate::rest::webhooks::types::SortDirection;





#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPermissionFE {
    pub id: String,
    pub resource_id: String,
    pub granted_to: String,
    pub granted_by: String,
    pub permission_types: Vec<SystemPermissionType>,
    pub begin_date_ms: i64,
    pub expiry_date_ms: i64,
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<String>,
    pub labels: Vec<LabelStringValue>,
    pub redeem_code: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
    
    // Additional FE-specific fields
    pub resource_name: Option<String>,
    pub grantee_name: Option<String>,
    pub grantee_avatar: Option<String>,
    pub granter_name: Option<String>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl SystemPermissionFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions {
                // redacted.system_permission.private_note = None;
            }
        }
        // Filter labels
        redacted.labels = match is_owner {
            true => redacted.labels,
            false => redacted.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryPermissionFE {
    pub id: String,
    pub resource_id: String,
    pub resource_path: DriveClippedFilePath,
    pub granted_to: String,
    pub granted_by: String,
    pub permission_types: Vec<DirectoryPermissionType>,
    pub begin_date_ms: i64,
    pub expiry_date_ms: i64,
    pub inheritable: bool,
    pub note: String,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_grantee: Option<String>,
    pub labels: Vec<LabelStringValue>,
    pub redeem_code: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    
    // Additional FE-specific fields
    pub resource_name: Option<String>,
    pub grantee_name: Option<String>,
    pub grantee_avatar: Option<String>,
    pub granter_name: Option<String>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl DirectoryPermissionFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {

            redacted.resource_path = DriveClippedFilePath("".to_string());

            // 2nd most sensitive
            if !has_edit_permissions {
                // redacted.system_permission.private_note = None;
            }
        }
        // Filter labels
        redacted.labels = match is_owner {
            true => redacted.labels,
            false => redacted.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirectoryPermissionsRequestBody {
    pub filters: ListDirectoryPermissionsRequestBodyFilters,
    pub page_size: Option<usize>,
    pub direction: Option<SortDirection>,
    pub cursor: Option<String>,
    // consider refactoring pagination to use "smart cursor" which is a string that has 3 parts `{resource_id}:{filter_index}:{global_index}`. this might be overcomplicating it and theres already established best practices
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirectoryPermissionsRequestBodyFilters {
    pub resource_id: String,
}


#[derive(Debug, Clone, Serialize)]
pub struct ListDirectoryPermissionsResponseData {
    pub items: Vec<DirectoryPermissionFE>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}
pub type ListDirectoryPermissionsResponse<'a> = ApiResponse<'a, ListDirectoryPermissionsResponseData>;





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
    pub id: Option<ClientSuggestedUUID>,
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

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::DirectoryPermission)?;
        }
        
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
    pub permission_types: Option<Vec<DirectoryPermissionType>>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub inheritable: Option<bool>,
    pub note: Option<String>,
    pub metadata: Option<PermissionMetadata>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl UpdateDirectoryPermissionsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        validate_id_string(&self.id.0, "id")?;

        
        // Validate permission_types (must not be empty)
        if let Some(perm_types) = &self.permission_types {
            if perm_types.is_empty() {
                return Err(ValidationError {
                    field: "permission_types".to_string(),
                    message: "Permission types cannot be empty".to_string(),
                });
            }
        }
        
        // Validate note if provided
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
        }
        
        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(&external_id.to_string())?;
        }
        
        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(&external_payload.to_string())?;
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
    pub resource_id: String,
    pub grantee_id: String,
    pub permissions: Vec<DirectoryPermissionType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedeemPermissionRequest {
    pub permission_id: String,
    pub user_id: String,
    pub redeem_code: String,
    pub note: Option<String>,
}

impl RedeemPermissionRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate permission_id
        validate_id_string(&self.permission_id, "permission_id")?;
        
        // Validate user_id
        validate_id_string(&self.user_id, "user_id")?;

        if (self.note.is_some()) {
            validate_description(self.note.as_ref().unwrap(), "note")?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedeemPermissionResponseData {
    pub permission: DirectoryPermissionFE,
}

pub type RedeemPermissionResponse<'a> = ApiResponse<'a, RedeemPermissionResponseData>;
pub type GetPermissionResponse<'a> = ApiResponse<'a, DirectoryPermissionFE>;
pub type CreatePermissionsResponse<'a> = ApiResponse<'a, CreateDirectoryPermissionsResponseData>;
pub type UpdatePermissionsResponse<'a> = ApiResponse<'a, UpdateDirectoryPermissionsResponseData>;
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


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListSystemPermissionsRequestBody {
    #[serde(default)]
    pub filters: ListSystemPermissionsRequestBodyFilters,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor: Option<String>,
    // consider refactoring pagination to use "smart cursor" which is a string that has 3 parts `{resource_id}:{filter_index}:{global_index}`. this might be overcomplicating it and theres already established best practices
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListSystemPermissionsRequestBodyFilters {
    pub resource_ids: Option<Vec<SystemResourceID>>, // leave this empty to get all permissions for all resources, but due to pagination we cant use .cast_fe().redact(), we must first filter out those the requesters does not have access to
    pub grantee_ids: Option<Vec<PermissionGranteeID>>, // leave this empty to get all permissions, but due to pagination we cant use .cast_fe().redact(), we must first filter out those the requesters does not have access to
    pub labels: Option<Vec<LabelStringValue>>, // leave this empty to get all permissions, but due to pagination we cant use .cast_fe().redact(), we must first filter out those the requesters does not have access to
}


#[derive(Debug, Clone, Serialize)]
pub struct ListSystemPermissionsResponseData {
    pub items: Vec<SystemPermissionFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor: Option<String>,
}
pub type ListSystemPermissionsResponse<'a> = ApiResponse<'a, ListSystemPermissionsResponseData>;



// Create System Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSystemPermissionsRequestBody {
    pub id: Option<ClientSuggestedUUID>,
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


        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::SystemPermission)?;
        }

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
            validate_external_id(&external_id.to_string())?;
        }
        
        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(&external_payload.to_string())?;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub resource_id: String,
    pub grantee_id: String,
    pub permissions: Vec<SystemPermissionType>,
}

// Redeem System Permission
#[derive(Debug, Clone, Deserialize)]
pub struct RedeemSystemPermissionRequest {
    pub permission_id: String,
    pub user_id: String,
    pub redeem_code: String,
    pub note: Option<String>,
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
pub type UpdateSystemPermissionsResponse<'a> = ApiResponse<'a, UpdateSystemPermissionsResponseData>;
pub type DeleteSystemPermissionResponse<'a> = ApiResponse<'a, DeleteSystemPermissionResponseData>;
pub type CheckSystemPermissionResponse<'a> = ApiResponse<'a, CheckSystemPermissionResult>;
pub type RedeemSystemPermissionResponse<'a> = ApiResponse<'a, RedeemSystemPermissionResponseData>;
