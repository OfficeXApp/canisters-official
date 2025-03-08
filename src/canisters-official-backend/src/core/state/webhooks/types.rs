// src/core/state/webhooks/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::{api::permissions::system::check_system_permissions, state::{directory::types::{FileID, FolderID}, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}}, types::{IDPrefix, UserID}};

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
    pub const FILE_CREATED: &'static str = "FILE_CREATED";
    pub const FOLDER_CREATED: &'static str = "FOLDER_CREATED";
    pub const RESTORE_TRASH: &'static str = "RESTORE_TRASH";
    pub const STATE_DIFFS: &'static str = "STATE_DIFFS";
    pub const SUPERSWAP_USER: &'static str = "SUPERSWAP_USER";

    // Helper method to create new instances
    pub fn new(id: String) -> Self {
        WebhookAltIndexID(id)
    }

    // Helper methods to get the constant instances
    pub fn file_created_slug() -> Self {
        WebhookAltIndexID(Self::FILE_CREATED.to_string())
    }

    pub fn folder_created_slug() -> Self {
        WebhookAltIndexID(Self::FOLDER_CREATED.to_string())
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
}



#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Webhook {
    pub id: WebhookID,
    pub url: String,
    pub alt_index: WebhookAltIndexID,
    pub event: WebhookEventLabel,
    pub signature: String,
    pub description: String,
    pub active: bool,
    pub filters: String,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
    pub created_at: u64,
}

impl Webhook {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        // let table_permissions = check_system_permissions(
        //     SystemResourceID::Table(SystemTableEnum::Webhooks),
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        // let permissions = check_system_permissions(
        //     resource_id,
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let has_edit_permissions = permissions.contains(&SystemPermissionType::Update) || table_permissions.contains(&SystemPermissionType::Update);

        // Filter tags
        redacted.tags = match is_owner {
            true => redacted.tags,
            false => redacted.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
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
    #[serde(rename = "team.invite.created")]
    TeamInviteCreated,
    #[serde(rename = "team.invite.updated")]
    TeamInviteUpdated,
    #[serde(rename = "drive.restore_trash")]
    DriveRestoreTrash,
    #[serde(rename = "drive.state_diffs")]
    DriveStateDiffs,
    #[serde(rename = "tag.added")]
    TagAdded,
    #[serde(rename = "tag.removed")]
    TagRemoved,
    #[serde(rename = "organization.superswap_user")]
    OrganizationSuperswapUser,
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
            "team.invite.created" => Ok(Self::TeamInviteCreated),
            "team.invite.updated" => Ok(Self::TeamInviteUpdated),
            "tag.added" => Ok(Self::TagAdded),
            "tag.removed" => Ok(Self::TagRemoved),
            "drive.restore_trash" => Ok(Self::DriveRestoreTrash),
            "drive.state_diffs" => Ok(Self::DriveStateDiffs),
            "organization.superswap_user" => Ok(Self::OrganizationSuperswapUser),
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
            // team
            Self::TeamInviteCreated => "team.invite.created",
            Self::TeamInviteUpdated => "team.invite.updated",
            // drive
            Self::DriveRestoreTrash => "drive.restore_trash",
            Self::DriveStateDiffs => "drive.state_diffs",
            // tags
            Self::TagAdded => "tag.added",
            Self::TagRemoved => "tag.removed",
            // organization
            Self::OrganizationSuperswapUser => "organization.superswap_user",
        }.to_string()
    }
}