// src/rest/permissions/types.rs
use serde::{Deserialize, Serialize};
use crate::core::state::permissions::types::*;
use crate::rest::directory::types::DirectoryResourceID;
use crate::rest::drives::types::DriveResponse;

// Response type included in FileMetadata/FolderMetadata
#[derive(Debug, Clone, Serialize)]
pub struct ResourcePermissionInfo {
    pub user_permissions: Vec<DirectoryPermissionType>,
    pub is_owner: bool,
    pub can_share: bool,
}

// Upsert Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct UpsertPermissionsRequest {
    pub permissions: Vec<PermissionEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionEntry {
    pub id: Option<DirectoryPermissionID>,
    pub resource_id: DirectoryResourceID,
    pub resource_path: String,
    pub grantee_type: DirectoryGranteeType,
    pub granted_to: DirectoryGranteeID,
    pub permission_types: Vec<DirectoryPermissionType>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub inheritable: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpsertPermissionsResponseData {
    pub permissions: Vec<DirectoryPermission>,
    pub failed_indices: Vec<(usize, String)>, // Index and error message for failed operations
}

// Delete Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct DeletePermissionsRequest {
    pub permission_ids: Vec<DirectoryPermissionID>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletePermissionsResponseData {
    pub deleted_ids: Vec<DirectoryPermissionID>,
    pub failed_deletions: Vec<(DirectoryPermissionID, String)>, // ID and error message for failed deletions
}

// Check Permissions
#[derive(Debug, Clone, Deserialize)]
pub struct PermissionCheckRequest {
    pub resource_id: DirectoryResourceID,
    pub grantee_id: DirectoryGranteeID,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckPermissionResult {
    pub resource_id: DirectoryResourceID,
    pub grantee_id: DirectoryGranteeID,
    pub permissions: Vec<DirectoryPermissionType>,
}

// Response type aliases using DriveResponse
pub type GetPermissionResponse<'a> = DriveResponse<'a, DirectoryPermission>;
pub type UpsertPermissionsResponse<'a> = DriveResponse<'a, UpsertPermissionsResponseData>;
pub type DeletePermissionsResponse<'a> = DriveResponse<'a, DeletePermissionsResponseData>;
pub type CheckPermissionResponse<'a> = DriveResponse<'a, CheckPermissionResult>;
pub type ErrorResponse<'a> = DriveResponse<'a, ()>;