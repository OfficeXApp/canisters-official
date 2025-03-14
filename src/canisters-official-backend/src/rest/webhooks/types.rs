// src/rest/webhooks/types.rs

use serde::{Deserialize, Serialize};
use crate::core::api::uuid::ShareTrackHash;
use crate::core::state::directory::types::{FileRecord, FolderRecord, ShareTrackID, ShareTrackResourceID};
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::drives::types::{DriveID, DriveRESTUrlEndpoint, StateChecksum, DriveStateDiffID, DriveStateDiffImplementationType, StateDiffRecord, DriveStateDiffString};
use crate::core::state::permissions::types::SystemPermissionType;
use crate::core::state::tags::state::validate_uuid4_string_with_prefix;
use crate::core::state::tags::types::{redact_tag, Tag, TagID, TagResourceID, TagStringValue};
use crate::core::state::group_invites::types::GroupInvite;
use crate::core::state::groups::types::Group;
use crate::core::state::webhooks::types::{WebhookAltIndexID, WebhookEventLabel};
use crate::core::state::webhooks::types::{WebhookID, Webhook};
use crate::core::types::{ClientSuggestedUUID, IDPrefix, UserID};
use crate::rest::directory::types::DirectoryResourcePermissionFE;
use crate::rest::types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_short_string, validate_unclaimed_uuid, validate_url_endpoint, ApiResponse, UpsertActionTypeEnum, ValidationError};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFE {
    #[serde(flatten)] 
    pub webhook: Webhook,
    pub permission_previews: Vec<SystemPermissionType>, 
}

impl WebhookFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {
            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.webhook.signature = "".to_string();
            }
        }
        // Filter tags
        redacted.webhook.tags = match is_owner {
            true => redacted.webhook.tags,
            false => redacted.webhook.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}




#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

impl ListWebhooksRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate filters string length (up to 256 chars)
        if self.filters.len() > 256 {
            return Err(ValidationError {
                field: "filters".to_string(),
                message: "Filters must be 256 characters or less".to_string(),
            });
        }

        // Validate page_size is reasonable
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(ValidationError {
                field: "page_size".to_string(),
                message: "Page size must be between 1 and 1000".to_string(),
            });
        }

        // Validate cursor strings if present
        if let Some(cursor) = &self.cursor_up {
            validate_short_string(cursor, "cursor_up")?;
        }

        if let Some(cursor) = &self.cursor_down {
            validate_short_string(cursor, "cursor_down")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListWebhooksResponseData {
    pub items: Vec<WebhookFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateWebhookRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub alt_index: String,
    pub url: String,
    pub event: String,
    pub signature: Option<String>,
    pub name: Option<String>,
    pub note: Option<String>,
    pub active: Option<bool>,
    pub filters: Option<String>, // filters is unsafe string from clients, any operations relying on filters should be wrapped in error handler
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateWebhookRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::Webhook)?;
        }
        
        // Validate alt_index
        validate_short_string(&self.alt_index, "alt_index")?;

        // Validate URL
        validate_url_endpoint(&self.url, "url")?;

        // Validate event
        validate_short_string(&self.event, "event")?;


        if let Some(name) = &self.name {
            validate_short_string(name, "name")?;
        }

        // Validate signature if provided
        if let Some(signature) = &self.signature {
            validate_short_string(signature, "signature")?;
        }

        if let Some(note) = &self.note {
            validate_description(note, "note")?;
        }

        // Validate filters if provided
        if let Some(filters) = &self.filters {
            if filters.len() > 256 {
                return Err(ValidationError {
                    field: "filters".to_string(),
                    message: "Filters must be 256 characters or less".to_string(),
                });
            }
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


#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebhookRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,   
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}
impl UpdateWebhookRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate webhook ID
        validate_id_string(&self.id, "id")?;
        
        // Validate URL if provided
        if let Some(url) = &self.url {
            validate_url_endpoint(url, "url")?;
        }
        
        // Validate signature if provided
        if let Some(signature) = &self.signature {
            validate_short_string(signature, "signature")?;
        }
        
        // Validate description if provided
        if let Some(name) = &self.name {
            validate_short_string(name, "name")?;
        }
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
        }
        
        // Validate filters if provided
        if let Some(filters) = &self.filters {
            if filters.len() > 256 {
                return Err(ValidationError {
                    field: "filters".to_string(),
                    message: "Filters must be 256 characters or less".to_string(),
                });
            }
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


#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWebhookRequest {
    pub id: String,
}
impl DeleteWebhookRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate webhook ID
        validate_id_string(&self.id, "id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedWebhookData {
    pub id: WebhookID,
    pub deleted: bool
}


pub type GetWebhookResponse<'a> = ApiResponse<'a, WebhookFE>;
pub type ListWebhooksResponse<'a> = ApiResponse<'a, ListWebhooksResponseData>;
pub type CreateWebhookResponse<'a> = ApiResponse<'a, WebhookFE>;
pub type UpdateWebhookResponse<'a> = ApiResponse<'a, WebhookFE>;
pub type DeleteWebhookResponse<'a> = ApiResponse<'a, DeletedWebhookData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;


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
    #[serde(rename = "group_invite")]
    GroupInvite(GroupInviteWebhookData),
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
    #[serde(rename = "tag")]
    Tag(TagWebhookData),
    #[serde(rename = "superswap_userid")]
    SuperswapUserID(UserID),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInviteWebhookData {
    pub group: Option<Group>,
    pub group_invite: Option<GroupInvite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWebhookData {
    pub resource_id: TagResourceID,
    pub tag_id: TagID,
    pub tag_value: TagStringValue,
    pub add: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveStateDiffWebhookData {
    pub data: StateDiffRecord
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
    pub endpoint_url: DriveRESTUrlEndpoint,
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
    pub file: Option<FileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderWebhookData {
    pub folder: Option<FolderRecord>,
}
