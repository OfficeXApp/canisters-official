// src/rest/webhooks/types.rs

use serde::{Deserialize, Serialize};
use crate::core::api::uuid::ShareTrackHash;
use crate::core::state::directory::types::{FileMetadata, FolderMetadata, ShareTrackID, ShareTrackResourceID};
use crate::core::state::drives::types::{DriveID, DriveRESTUrlEndpoint, DriveStateDiffChecksum, DriveStateDiffID, DriveStateDiffImplementationType, DriveStateDiffRecord, DriveStateDiffString};
use crate::core::state::team_invites::types::Team_Invite;
use crate::core::state::teams::types::Team;
use crate::core::state::webhooks::types::{WebhookAltIndexID, WebhookEventLabel};
use crate::core::state::webhooks::types::{WebhookID, Webhook};
use crate::core::types::UserID;
use crate::rest::directory::types::DirectoryResourcePermissionFE;

#[derive(Debug, Clone, Serialize)]
pub enum WebhookResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> WebhookResponse<'a, T> {
    pub fn ok(data: &'a T) -> WebhookResponse<'a, T> { 
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn unauthorized() -> Self {
        Self::err(401, "Unauthorized".to_string())
    }

    pub fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize value")
    }
}


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}



#[derive(Debug, Clone, Deserialize)]
pub struct ListWebhooksRequestBody {
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

fn default_page_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize)]
pub struct ListWebhooksResponseData {
    pub items: Vec<Webhook>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateWebhookRequestBody {
    pub alt_index: String,
    pub url: String,
    pub event: String,
    pub signature: Option<String>,
    pub description: Option<String>,
    pub filters: Option<String> // filters is unsafe string from clients, any operations relying on filters should be wrapped in error handler
}


#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebhookRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,   
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertWebhookRequestBody {
    Create(CreateWebhookRequestBody),
    Update(UpdateWebhookRequestBody),
}


#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWebhookRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedWebhookData {
    pub id: WebhookID,
    pub deleted: bool
}


pub type GetWebhookResponse<'a> = WebhookResponse<'a, Webhook>;
pub type ListWebhooksResponse<'a> = WebhookResponse<'a, ListWebhooksResponseData>;
pub type CreateWebhookResponse<'a> = WebhookResponse<'a, Webhook>;
pub type UpdateWebhookResponse<'a> = WebhookResponse<'a, Webhook>;
pub type DeleteWebhookResponse<'a> = WebhookResponse<'a, DeletedWebhookData>;
pub type ErrorResponse<'a> = WebhookResponse<'a, ()>;


/**
 * 
 * Webhook Event Payloads
 * 
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventPayload {
    pub event: String,
    pub timestamp_ms: u64,
    pub nonce: u64,
    pub webhook_id: WebhookID,
    pub webhook_alt_index: WebhookAltIndexID,
    pub payload: WebhookEventData,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventData {
    pub before: Option<WebhookResourceData>,
    pub after: Option<WebhookResourceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebhookResourceData {
    #[serde(rename = "team_invite")]
    TeamInvite(TeamInviteWebhookData),
    #[serde(rename = "file")]
    File(FileWebhookData),
    #[serde(rename = "folder")]
    Folder(FolderWebhookData),
    #[serde(rename = "subfile")]
    Subfile(FileWebhookData),
    #[serde(rename = "subfolder")]
    Subfolder(FolderWebhookData),
    #[serde(rename = "share_tracking")]
    ShareTracking(ShareTrackingWebhookData),
    #[serde(rename = "state_diffs")]
    StateDiffs(DriveStateDiffWebhookData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInviteWebhookData {
    pub team: Option<Team>,
    pub team_invite: Option<Team_Invite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveStateDiffWebhookData {
    pub data: DriveStateDiffRecord
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareTrackingWebhookData {
    pub id: ShareTrackID,
    pub hash: ShareTrackHash,
    pub origin_id: Option<ShareTrackID>,
    pub origin_hash: Option<ShareTrackHash>,
    pub from_user: Option<UserID>,
    pub to_user: Option<UserID>,
    pub resource_id: ShareTrackResourceID,
    pub resource_name: String,
    pub drive_id: DriveID,
    pub timestamp_ms: u64,
    pub url_endpoint: DriveRESTUrlEndpoint,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectoryWebhookData {
    File(FileWebhookData),
    Folder(FolderWebhookData),
    Subfile(FileWebhookData),
    Subfolder(FolderWebhookData),
    ShareTracking(ShareTrackingWebhookData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWebhookData {
    pub file: Option<FileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderWebhookData {
    pub folder: Option<FolderMetadata>,
}
