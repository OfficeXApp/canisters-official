// src/core/state/labels/types.rs

use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;

use crate::{core::{
    api::permissions::system::{check_system_permissions, check_system_resource_permissions_labels}, state::{
        api_keys::types::ApiKeyID,
        contacts::types::Contact,
        directory::types::{FileID, FolderID},
        disks::types::DiskID,
        drives::{state::state::OWNER_ID, types::{DriveID, ExternalID, ExternalPayload}},
        permissions::types::{DirectoryPermissionID, PermissionGranteeID, SystemPermissionID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum},
        group_invites::types::GroupInviteID,
        groups::types::GroupID,
        webhooks::types::WebhookID
    }, types::{IDPrefix, UserID}
}, rest::{contacts::types::ContactGroupInvitePreview, labels::types::LabelFE}};

use super::state::LABELS_BY_VALUE_HASHTABLE;

// LabelID is the unique identifier for a label
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct LabelID(pub String);

impl fmt::Display for LabelID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// LabelStringValue is the actual text of the label
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct LabelStringValue(pub String);

impl fmt::Display for LabelStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// HexColorString represents a color in hex format
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct HexColorString(pub String);

impl fmt::Display for HexColorString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// The main Label type that represents a label definition
// We also dont redact labels here, for convinience. if we find this is a security issue, we can redact labels here too
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Label {
    pub id: LabelID,
    pub value: LabelStringValue,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub color: HexColorString,
    pub created_by: UserID, // wont get updated by superswap, reverse lookup HISTORY_SUPERSWAP_USERID
    pub created_at: u64,
    pub last_updated_at: u64,
    pub resources: Vec<LabelResourceID>,
    pub labels: Vec<LabelStringValue>,  // Labels can be labelged too
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Label {

    pub fn cast_fe(&self, user_id: &UserID) -> LabelFE {
        let label = self.clone();
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Label(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Labels),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        LabelFE {
            label,
            permission_previews
        }.redacted(user_id)
    }

    
}


// LabelResourceID represents any resource that can be labelged
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum LabelResourceID {
    ApiKey(ApiKeyID),
    Contact(UserID),
    File(FileID),
    Folder(FolderID),
    Disk(DiskID),
    Drive(DriveID),
    DirectoryPermission(DirectoryPermissionID),
    SystemPermission(SystemPermissionID),
    GroupInvite(GroupInviteID),
    Group(GroupID),
    Webhook(WebhookID),
    Label(LabelID),
}

impl fmt::Display for LabelResourceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelResourceID::ApiKey(id) => write!(f, "{}", id),
            LabelResourceID::Contact(id) => write!(f, "{}", id),
            LabelResourceID::File(id) => write!(f, "{}", id),
            LabelResourceID::Folder(id) => write!(f, "{}", id),
            LabelResourceID::Disk(id) => write!(f, "{}", id),
            LabelResourceID::Drive(id) => write!(f, "{}", id),
            LabelResourceID::DirectoryPermission(id) => write!(f, "{}", id),
            LabelResourceID::SystemPermission(id) => write!(f, "{}", id),
            LabelResourceID::GroupInvite(id) => write!(f, "{}", id),
            LabelResourceID::Group(id) => write!(f, "{}", id),
            LabelResourceID::Webhook(id) => write!(f, "{}", id),
            LabelResourceID::Label(id) => write!(f, "{}", id),
        }
    }
}

impl LabelResourceID {
    pub fn get_id_string(&self) -> String {
        match self {
            LabelResourceID::ApiKey(id) => id.0.clone(),
            LabelResourceID::Contact(id) => id.0.clone(),
            LabelResourceID::File(id) => id.0.clone(),
            LabelResourceID::Folder(id) => id.0.clone(),
            LabelResourceID::Disk(id) => id.0.clone(),
            LabelResourceID::Drive(id) => id.0.clone(),
            LabelResourceID::DirectoryPermission(id) => id.0.clone(),
            LabelResourceID::SystemPermission(id) => id.0.clone(),
            LabelResourceID::GroupInvite(id) => id.0.clone(),
            LabelResourceID::Group(id) => id.0.clone(),
            LabelResourceID::Webhook(id) => id.0.clone(),
            LabelResourceID::Label(id) => id.0.clone(),
        }
    }
}

// Request and response types for label operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLabelRequest {
    pub value: String,
    pub description: Option<String>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLabelRequest {
    pub id: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpsertLabelRequest {
    Create(CreateLabelRequest),
    Update(UpdateLabelRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelResourceRequest {
    pub label_id: String,
    pub resource_id: String,
    pub add: bool,  // true to add, false to remove
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelOperationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub label: Option<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLabelsRequest {
    pub query: Option<String>,
    pub page_size: Option<usize>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListLabelsResponse {
    pub items: Vec<Label>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteLabelRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteLabelResponse {
    pub success: bool,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLabelResourcesRequest {
    pub label_id: String,
    pub resource_type: Option<String>,
    pub page_size: Option<usize>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetLabelResourcesResponse {
    pub label_id: String,
    pub resources: Vec<LabelResourceID>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub fn redact_label(label_value: LabelStringValue, user_id: UserID) -> Option<LabelStringValue> {
    // Get the label ID from the value
    let label_id = LABELS_BY_VALUE_HASHTABLE.with(|store| {
        store.borrow().get(&label_value).cloned()
    });
    
    if let Some(label_id) = label_id {
        // Check if the user is the owner
        let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow());
        
        if is_owner {
            // Owner sees everything, no redaction needed
            return Some(label_value);
        }
        
        // Check permissions for this specific label
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Label(label_id.to_string()));
        let permissions = check_system_resource_permissions_labels(
            &resource_id,
            &PermissionGranteeID::User(user_id.clone()),
            &label_value.to_string()
        );
        
        // Check permissions for the Labels table
        let table_permissions = check_system_resource_permissions_labels(
            &SystemResourceID::Table(SystemTableEnum::Labels),
            &PermissionGranteeID::User(user_id.clone()),
            &label_value.to_string()
        );
        
        // If the user has View permission either at the table level or for this specific label
        if permissions.contains(&SystemPermissionType::View) || table_permissions.contains(&SystemPermissionType::View) {
            return Some(label_value);
        }

        // Check if there are any permissions with label prefixes that would allow viewing
        // (This is already handled by check_system_resource_permissions_labels)
        
        // If we get here, the user doesn't have permission to see this label
        return None;
    }
    
    // Label not found, so we can't provide it
    None
}

pub fn redact_group_previews(group_preview: ContactGroupInvitePreview, user_id: UserID) -> Option<ContactGroupInvitePreview> {
    // Get the group ID from the preview
    let group_id = &group_preview.group_id;
    
    // Check if the user is the owner
    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow());
    
    if is_owner {
        // Owner sees everything, no redaction needed
        return Some(group_preview);
    }
    
    // Check permissions for this specific group
    let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Group(group_id.to_string()));
    let permissions = check_system_permissions(
        resource_id,
        PermissionGranteeID::User(user_id.clone())
    );
    
    // Check permissions for the Groups table
    let table_permissions = check_system_permissions(
        SystemResourceID::Table(SystemTableEnum::Groups),
        PermissionGranteeID::User(user_id.clone())
    );

    let group = match crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE
        .with(|groups| groups.borrow().get(group_id).cloned()) {
        Some(group) => group,
        None => return None
    };
    
    // Check if user is a member of this group
    let is_group_member = crate::core::state::groups::state::state::is_user_on_local_group(&user_id, &group);
    
    // If the user has View permission either at the table level or for this specific group
    // or if the user is a member of the group
    if permissions.contains(&SystemPermissionType::View) || 
       table_permissions.contains(&SystemPermissionType::View) ||
       is_group_member {
        return Some(group_preview);
    }
    
    // If we get here, the user doesn't have permission to see this group
    None
}