// src/core/state/webhooks/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::{core::{api::permissions::system::check_system_permissions, state::{directory::types::{FileID, FolderID}, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, labels::types::{redact_label, LabelStringValue}}, types::{IDPrefix, UserID}}, rest::webhooks::types::WebhookFE};



#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Webhook {
    pub id: WebhookID,
    pub name: String,
    pub url: String,
    pub alt_index: WebhookAltIndexID,
    pub event: WebhookEventLabel,
    pub signature: String,
    pub note: Option<String>,
    pub active: bool,
    pub filters: String,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
    pub created_at: u64,
}

impl Webhook {

    pub fn cast_fe(&self, user_id: &UserID) -> WebhookFE {
        let webhook = self.clone();
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Webhook(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Webhooks),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        WebhookFE {
            webhook,
            permission_previews
        }.redacted(user_id)
    }

    
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct WebhookID(pub String);
impl fmt::Display for WebhookID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct WebhookAltIndexID(pub String);
impl fmt::Display for WebhookAltIndexID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WebhookAltIndexID {
    pub const ALL_FILES: &'static str = "ALL_FILES";
    pub const ALL_FOLDERS: &'static str = "ALL_FOLDERS";
    pub const RESTORE_TRASH: &'static str = "RESTORE_TRASH"; // this alt index is required due to it querying directory
    pub const STATE_DIFFS: &'static str = "STATE_DIFFS"; 
    pub const SUPERSWAP_USER: &'static str = "SUPERSWAP_USER";
    pub const INBOX_NEW_MAIL: &'static str = "INBOX_NEW_MAIL";

    // Helper method to create new instances
    pub fn new(id: String) -> Self {
        WebhookAltIndexID(id)
    }

    // Helper methods to get the constant instances
    pub fn file_created_slug() -> Self {
        WebhookAltIndexID(Self::ALL_FILES.to_string())
    }

    pub fn folder_created_slug() -> Self {
        WebhookAltIndexID(Self::ALL_FOLDERS.to_string())
    }

    pub fn restore_trash_slug() -> Self {
        WebhookAltIndexID(Self::RESTORE_TRASH.to_string())
    }

    pub fn state_diffs_slug() -> Self {
        WebhookAltIndexID(Self::STATE_DIFFS.to_string())
    }

    pub fn superswap_user_slug() -> Self {
        WebhookAltIndexID(Self::SUPERSWAP_USER.to_string())
    }

    pub fn inbox_new_notif_slug() -> Self {
        WebhookAltIndexID(Self::INBOX_NEW_MAIL.to_string())
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventLabel {
    #[serde(rename = "file.viewed")]
    FileViewed,
    #[serde(rename = "file.created")]
    FileCreated,
    #[serde(rename = "file.updated")]
    FileUpdated,
    #[serde(rename = "file.deleted")]
    FileDeleted,
    #[serde(rename = "file.shared")]
    FileShared,
    #[serde(rename = "folder.viewed")]
    FolderViewed,
    #[serde(rename = "folder.created")]
    FolderCreated,
    #[serde(rename = "folder.updated")]
    FolderUpdated,
    #[serde(rename = "folder.deleted")]
    FolderDeleted,
    #[serde(rename = "folder.shared")]
    FolderShared,
    #[serde(rename = "subfile.viewed")]
    SubfileViewed,
    #[serde(rename = "subfile.created")]
    SubfileCreated,
    #[serde(rename = "subfile.updated")]
    SubfileUpdated,
    #[serde(rename = "subfile.deleted")]
    SubfileDeleted,
    #[serde(rename = "subfile.shared")]
    SubfileShared,
    #[serde(rename = "subfolder.viewed")]
    SubfolderViewed,
    #[serde(rename = "subfolder.created")]
    SubfolderCreated,
    #[serde(rename = "subfolder.updated")]
    SubfolderUpdated,
    #[serde(rename = "subfolder.deleted")]
    SubfolderDeleted,
    #[serde(rename = "subfolder.shared")]
    SubfolderShared,
    #[serde(rename = "group.invite.created")]
    GroupInviteCreated,
    #[serde(rename = "group.invite.updated")]
    GroupInviteUpdated,
    #[serde(rename = "drive.restore_trash")]
    DriveRestoreTrash,
    #[serde(rename = "drive.state_diffs")]
    DriveStateDiffs,
    #[serde(rename = "label.added")]
    LabelAdded,
    #[serde(rename = "label.removed")]
    LabelRemoved,
    #[serde(rename = "org.superswap_user")]
    OrganizationSuperswapUser,
    #[serde(rename = "org.inbox.new_mail")]
    OrganizationInboxNewNotif,
}

impl std::str::FromStr for WebhookEventLabel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file.viewed" => Ok(Self::FileViewed),
            "file.created" => Ok(Self::FileCreated),
            "file.updated" => Ok(Self::FileUpdated),
            "file.deleted" => Ok(Self::FileDeleted),
            "file.shared" => Ok(Self::FileShared),
            "folder.viewed" => Ok(Self::FolderViewed),
            "folder.created" => Ok(Self::FolderCreated),
            "folder.updated" => Ok(Self::FolderUpdated),
            "folder.deleted" => Ok(Self::FolderDeleted),
            "folder.shared" => Ok(Self::FolderShared),
            "subfile.viewed" => Ok(Self::SubfileViewed),
            "subfile.created" => Ok(Self::SubfileCreated),
            "subfile.updated" => Ok(Self::SubfileUpdated),
            "subfile.deleted" => Ok(Self::SubfileDeleted),
            "subfile.shared" => Ok(Self::SubfileShared),
            "subfolder.viewed" => Ok(Self::SubfolderViewed),
            "subfolder.created" => Ok(Self::SubfolderCreated),
            "subfolder.updated" => Ok(Self::SubfolderUpdated),
            "subfolder.deleted" => Ok(Self::SubfolderDeleted),
            "subfolder.shared" => Ok(Self::SubfolderShared),
            "group.invite.created" => Ok(Self::GroupInviteCreated),
            "group.invite.updated" => Ok(Self::GroupInviteUpdated),
            "label.added" => Ok(Self::LabelAdded),
            "label.removed" => Ok(Self::LabelRemoved),
            "drive.restore_trash" => Ok(Self::DriveRestoreTrash),
            "drive.state_diffs" => Ok(Self::DriveStateDiffs),
            "org.superswap_user" => Ok(Self::OrganizationSuperswapUser),
            "org.inbox.new_mail" => Ok(Self::OrganizationInboxNewNotif),
            _ => Err(format!("Invalid webhook event: {}", s)),
        }
    }
}

// Optionally, if you need to convert back to string representation
impl ToString for WebhookEventLabel {
    fn to_string(&self) -> String {
        match self {
            // file
            Self::FileViewed => "file.viewed",
            Self::FileCreated => "file.created",
            Self::FileUpdated => "file.updated",
            Self::FileDeleted => "file.deleted",
            Self::FileShared => "file.shared",
            // folder
            Self::FolderViewed => "folder.viewed",
            Self::FolderCreated => "folder.created",
            Self::FolderUpdated => "folder.updated",
            Self::FolderDeleted => "folder.deleted",
            Self::FolderShared => "folder.shared",
            // subfile
            Self::SubfileViewed => "subfile.viewed",
            Self::SubfileCreated => "subfile.created",
            Self::SubfileUpdated => "subfile.updated",
            Self::SubfileDeleted => "subfile.deleted",
            Self::SubfileShared => "subfile.shared",
            // subfolder
            Self::SubfolderViewed => "subfolder.viewed",
            Self::SubfolderCreated => "subfolder.created",
            Self::SubfolderUpdated => "subfolder.updated",
            Self::SubfolderDeleted => "subfolder.deleted",
            Self::SubfolderShared => "subfolder.shared",
            // group
            Self::GroupInviteCreated => "group.invite.created",
            Self::GroupInviteUpdated => "group.invite.updated",
            // drive
            Self::DriveRestoreTrash => "drive.restore_trash",
            Self::DriveStateDiffs => "drive.state_diffs",
            // labels
            Self::LabelAdded => "label.added",
            Self::LabelRemoved => "label.removed",
            // organization
            Self::OrganizationSuperswapUser => "organization.superswap_user",
            Self::OrganizationInboxNewNotif => "organization.inbox.new_mail",
        }.to_string()
    }
}