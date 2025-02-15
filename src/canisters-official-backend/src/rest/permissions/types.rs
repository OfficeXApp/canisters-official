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
pub struct UpsertPermissionsRequestBody {
    pub id: Option<DirectoryPermissionID>,
    pub resource_id: String,
    pub granted_to: Option<String>,
    pub permission_types: Vec<DirectoryPermissionType>,
    pub begin_date_ms: Option<i64>,
    pub expiry_date_ms: Option<i64>,
    pub inheritable: bool,
    pub note: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct RedeemPermissionResponseData {
    pub permission: DirectoryPermission,
}

pub type RedeemPermissionResponse<'a> = DriveResponse<'a, RedeemPermissionResponseData>;


// Response type aliases using DriveResponse
pub type GetPermissionResponse<'a> = DriveResponse<'a, DirectoryPermission>;
pub type UpsertPermissionsResponse<'a> = DriveResponse<'a, UpsertPermissionsResponseData>;
pub type DeletePermissionResponse<'a> = DriveResponse<'a, DeletePermissionResponseData>;
pub type CheckPermissionResponse<'a> = DriveResponse<'a, CheckPermissionResult>;
pub type ErrorResponse<'a> = DriveResponse<'a, ()>;



// Get System Permission
#[derive(Debug, Clone, Deserialize)] 
pub struct GetSystemPermissionRequest {
    pub permission_id: SystemPermissionID,
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

#[derive(Debug, Clone, Serialize)]
pub struct RedeemSystemPermissionResponseData {
    pub permission: SystemPermission,
}

// Response type aliases
pub type GetSystemPermissionResponse<'a> = DriveResponse<'a, SystemPermission>;
pub type UpsertSystemPermissionsResponse<'a> = DriveResponse<'a, UpsertSystemPermissionsResponseData>;
pub type DeleteSystemPermissionResponse<'a> = DriveResponse<'a, DeleteSystemPermissionResponseData>;
pub type CheckSystemPermissionResponse<'a> = DriveResponse<'a, CheckSystemPermissionResult>;
pub type RedeemSystemPermissionResponse<'a> = DriveResponse<'a, RedeemSystemPermissionResponseData>;
