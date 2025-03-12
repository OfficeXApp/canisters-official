// src/rest/team_invites/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::{state::{drives::state::state::OWNER_ID, permissions::types::SystemPermissionType, tags::{state::validate_uuid4_string_with_prefix, types::redact_tag}, team_invites::types::{ TeamInviteID, TeamRole, TeamInvite}, teams::state::state::is_team_admin}, types::{ClientSuggestedUUID, IDPrefix, UserID}}, rest::{types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInviteFE {
    #[serde(flatten)] // this lets us "extend" the Contact struct
    pub team_invite: TeamInvite,
    pub team_name: String,
    pub team_avatar: Option<String>,
    pub invitee_name: String,
    pub invitee_avatar: Option<String>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl TeamInviteFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);
        let is_team_admin = is_team_admin(user_id, &self.team_invite.team_id);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions && !is_team_admin {
                redacted.team_invite.from_placeholder_invitee = None;
                redacted.team_invite.inviter_id = UserID("".to_string());
            }
        }
        // Filter tags
        redacted.team_invite.tags = match is_owner {
            true => redacted.team_invite.tags,
            false => redacted.team_invite.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}

// Update CreateTeam_InviteRequest in rest/team_invites/types.rs
#[derive(Debug, Clone, Deserialize)]
pub struct ListTeamInvitesRequestBody {
    pub team_id: String,
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

impl ListTeamInvitesRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate team_id
        validate_id_string(&self.team_id, "team_id")?;
        
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
        if let Some(cursor) = &self.cursor_up {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_up".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        if let Some(cursor) = &self.cursor_down {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_down".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListTeamInvitesResponseData {
    pub items: Vec<TeamInviteFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTeamInviteRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub team_id: String,
    pub invitee_id: Option<String>,
    pub role: TeamRole,
    pub active_from: Option<u64>,
    pub expires_at: Option<i64>,
    pub note: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl CreateTeamInviteRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {


        if self.id.is_some() {
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::TeamInvite)?;
        }
        
        // Validate team_id
        validate_id_string(&self.team_id, "team_id")?;
        
        // Validate invitee_id if present
        if let Some(invitee_id) = &self.invitee_id {
            validate_user_id(invitee_id)?;
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
pub struct UpdateTeamInviteRequestBody {
    pub id: TeamInviteID,
    pub role: Option<TeamRole>,
    pub active_from: Option<u64>,
    pub expires_at: Option<i64>,
    pub note: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl UpdateTeamInviteRequestBody {
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



pub type GetTeam_InviteResponse<'a> = ApiResponse<'a, TeamInviteFE>;

pub type ListTeam_InvitesResponse<'a> = ApiResponse<'a, ListTeamInvitesResponseData>;


pub type CreateTeam_InviteResponse<'a> = ApiResponse<'a, TeamInviteFE>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeam_InviteRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateTeam_InviteResponse<'a> = ApiResponse<'a, TeamInviteFE>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTeam_InviteRequest {
    pub id: TeamInviteID,
}
impl DeleteTeam_InviteRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id
        validate_id_string(&self.id.0, "id")?;
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DeletedTeam_InviteData {
    pub id: TeamInviteID,
    pub deleted: bool
}

pub type DeleteTeam_InviteResponse<'a> = ApiResponse<'a, DeletedTeam_InviteData>;


pub type ErrorResponse<'a> = ApiResponse<'a, ()>;

#[derive(Debug, Clone, Deserialize)]
pub struct RedeemTeamInviteRequest {
    pub invite_id: String,
    pub user_id: String,
}
impl RedeemTeamInviteRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate invite_id
        validate_id_string(&self.invite_id, "invite_id")?;
        
        // Validate user_id
        validate_user_id(&self.user_id)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedeemTeamInviteResponseData {
    pub invite: TeamInviteFE,
}