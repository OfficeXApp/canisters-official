// src/rest/group_invites/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::{state::{drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::SystemPermissionType, labels::{state::validate_uuid4_string_with_prefix, types::{redact_label, LabelStringValue}}, group_invites::types::{ GroupInvite, GroupInviteID, GroupRole}, groups::{state::state::is_group_admin, types::GroupID}}, types::{ClientSuggestedUUID, IDPrefix, UserID}}, rest::{types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_unclaimed_uuid, validate_user_id, ApiResponse, ValidationError}, webhooks::types::SortDirection}};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInviteFE {
    // Keep all fields from GroupInvite except invitee_id modified to String instead of enum label format
    pub id: GroupInviteID,
    pub group_id: GroupID,
    pub inviter_id: UserID,
    // Override with the flat string version
    pub invitee_id: String,
    pub role: GroupRole,
    pub note: String,
    pub active_from: u64,
    pub expires_at: i64,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_invitee: Option<String>,
    pub labels: Vec<LabelStringValue>,
    pub redeem_code: Option<String>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
    
    // Additional FE-specific fields
    pub group_name: String,
    pub group_avatar: Option<String>,
    pub invitee_name: String,
    pub invitee_avatar: Option<String>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl GroupInviteFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);
        let is_group_admin = is_group_admin(user_id, &self.group_id);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions && !is_group_admin {
                redacted.from_placeholder_invitee = None;
                redacted.inviter_id = UserID("".to_string());
                redacted.redeem_code = None;
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

// Update CreateGroup_InviteRequest in rest/group_invites/types.rs
#[derive(Debug, Clone, Deserialize)]
pub struct ListGroupInvitesRequestBody {
    pub group_id: String,
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

fn default_page_size() -> usize {
    50
}

impl ListGroupInvitesRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate group_id
        validate_id_string(&self.group_id, "group_id")?;
        
        // Validate filters string length
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
        if let Some(cursor) = &self.cursor {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }


        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListGroupInvitesResponseData {
    pub items: Vec<GroupInviteFE>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGroupInviteRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub group_id: String,
    pub invitee_id: Option<String>,
    pub role: Option<GroupRole>,
    pub active_from: Option<u64>,
    pub expires_at: Option<i64>,
    pub note: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl CreateGroupInviteRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {


        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::GroupInvite)?;
        }
        
        // Validate group_id
        validate_id_string(&self.group_id, "group_id")?;
        
        // Validate invitee_id if present and not PUBLIC
        match &self.invitee_id {
            Some(invitee_id) => {
                if invitee_id != "PUBLIC" {
                    validate_user_id(invitee_id)?;
                }
            },
            None => {}
        }

        
        // Validate note if present (description field)
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
        }
        
        // Validate external_id if present
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }
        
        // Validate external_payload if present
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGroupInviteRequestBody {
    pub id: GroupInviteID,
    pub role: Option<GroupRole>,
    pub active_from: Option<u64>,
    pub expires_at: Option<i64>,
    pub note: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl UpdateGroupInviteRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;
        
        // Validate note if present (description field)
        if let Some(note) = &self.note {
            validate_description(note, "note")?;
        }
        
        // Validate external_id if present
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }
        
        // Validate external_payload if present
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }
        
        Ok(())
    }
}



pub type GetGroup_InviteResponse<'a> = ApiResponse<'a, GroupInviteFE>;

pub type ListGroup_InvitesResponse<'a> = ApiResponse<'a, ListGroupInvitesResponseData>;


pub type CreateGroup_InviteResponse<'a> = ApiResponse<'a, GroupInviteFE>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGroup_InviteRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateGroup_InviteResponse<'a> = ApiResponse<'a, GroupInviteFE>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteGroup_InviteRequest {
    pub id: GroupInviteID,
}
impl DeleteGroup_InviteRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DeletedGroup_InviteData {
    pub id: GroupInviteID,
    pub deleted: bool
}

pub type DeleteGroup_InviteResponse<'a> = ApiResponse<'a, DeletedGroup_InviteData>;


pub type ErrorResponse<'a> = ApiResponse<'a, ()>;

#[derive(Debug, Clone, Deserialize)]
pub struct RedeemGroupInviteRequest {
    pub invite_id: String,
    pub user_id: String,
    pub redeem_code: String,
    pub note: Option<String>,
}
impl RedeemGroupInviteRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate invite_id
        validate_id_string(&self.invite_id, "invite_id")?;
        
        // Validate user_id
        validate_user_id(&self.user_id)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedeemGroupInviteResponseData {
    pub invite: GroupInviteFE,
}