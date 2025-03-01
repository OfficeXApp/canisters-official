// src/rest/team_invites/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::state::team_invites::types::{ TeamInviteID, TeamRole, Team_Invite}, rest::webhooks::types::SortDirection, types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_user_id, ValidationError}};



#[derive(Debug, Clone, Serialize)]
pub enum Team_InviteResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> Team_InviteResponse<'a, T> {
    pub fn ok(data: &'a T) -> Team_InviteResponse<'a, T> {
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
    pub items: Vec<Team_Invite>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTeamInviteRequestBody {
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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertTeamInviteRequestBody {
    Create(CreateTeamInviteRequestBody),
    Update(UpdateTeamInviteRequestBody),
}

impl UpsertTeamInviteRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertTeamInviteRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertTeamInviteRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}


pub type GetTeam_InviteResponse<'a> = Team_InviteResponse<'a, Team_Invite>;

pub type ListTeam_InvitesResponse<'a> = Team_InviteResponse<'a, ListTeamInvitesResponseData>;


pub type CreateTeam_InviteResponse<'a> = Team_InviteResponse<'a, Team_Invite>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeam_InviteRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateTeam_InviteResponse<'a> = Team_InviteResponse<'a, Team_Invite>;

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

pub type DeleteTeam_InviteResponse<'a> = Team_InviteResponse<'a, DeletedTeam_InviteData>;


pub type ErrorResponse<'a> = Team_InviteResponse<'a, ()>;

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
    pub invite: Team_Invite,
}