// src/core/state/webhooks/types.rs
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::core::{state::directory::types::{FileUUID, FolderUUID}, types::IDPrefix};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebhookID(pub String);
impl fmt::Display for WebhookID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: WebhookID,
    pub url: String,
    pub alt_index: WebhookAltIndexID,
    pub event: WebhookEventLabel,
    pub signature: String,
    pub description: String,
    pub active: bool,
    pub filters: String,
}



#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            "drive.restore_trash" => Ok(Self::DriveRestoreTrash),
            "drive.state_diffs" => Ok(Self::DriveStateDiffs),
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
        }.to_string()
    }
}