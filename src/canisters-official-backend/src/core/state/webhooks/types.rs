use std::fmt;

// src/core/state/webhooks/types.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebhookID(pub String);
impl fmt::Display for WebhookID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WebhookAltIndexID(pub String);

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
    #[serde(rename = "folder.file.created")]
    FolderFileCreated,
    #[serde(rename = "folder.file.updated")]
    FolderFileUpdated,
    #[serde(rename = "folder.file.deleted")]
    FolderFileDeleted,
    #[serde(rename = "folder.file.shared")]
    FolderFileShared,
    #[serde(rename = "team.invite.created")]
    TeamInviteCreated,
    #[serde(rename = "team.invite.updated")]
    TeamInviteUpdated,
    #[serde(rename = "drive.gas_low")]
    DriveGasLow,
    #[serde(rename = "drive.sync_completed")]
    DriveSyncCompleted,
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
            "folder.file.created" => Ok(Self::FolderFileCreated),
            "folder.file.updated" => Ok(Self::FolderFileUpdated),
            "folder.file.deleted" => Ok(Self::FolderFileDeleted),
            "folder.file.shared" => Ok(Self::FolderFileShared),
            "team.invite.created" => Ok(Self::TeamInviteCreated),
            "team.invite.updated" => Ok(Self::TeamInviteUpdated),
            "drive.gas_low" => Ok(Self::DriveGasLow),
            "drive.sync_completed" => Ok(Self::DriveSyncCompleted),
            _ => Err(format!("Invalid webhook event: {}", s)),
        }
    }
}

// Optionally, if you need to convert back to string representation
impl ToString for WebhookEventLabel {
    fn to_string(&self) -> String {
        match self {
            Self::FileViewed => "file.viewed",
            Self::FileCreated => "file.created",
            Self::FileUpdated => "file.updated",
            Self::FileDeleted => "file.deleted",
            Self::FileShared => "file.shared",
            Self::FolderViewed => "folder.viewed",
            Self::FolderCreated => "folder.created",
            Self::FolderUpdated => "folder.updated",
            Self::FolderDeleted => "folder.deleted",
            Self::FolderShared => "folder.shared",
            Self::FolderFileCreated => "folder.file.created",
            Self::FolderFileUpdated => "folder.file.updated",
            Self::FolderFileDeleted => "folder.file.deleted",
            Self::FolderFileShared => "folder.file.shared",
            Self::TeamInviteCreated => "team.invite.created",
            Self::TeamInviteUpdated => "team.invite.updated",
            Self::DriveGasLow => "drive.gas_low",
            Self::DriveSyncCompleted => "drive.sync_completed",
        }.to_string()
    }
}