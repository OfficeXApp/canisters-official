// src/rest/team_invites/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::state::team_invites::types::{ TeamInviteID, TeamRole, Team_Invite}, rest::webhooks::types::SortDirection};



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
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamInviteRequestBody {
    pub id: TeamInviteID,
    pub role: Option<TeamRole>,
    pub active_from: Option<u64>,
    pub expires_at: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertTeamInviteRequestBody {
    Create(CreateTeamInviteRequestBody),
    Update(UpdateTeamInviteRequestBody),
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

#[derive(Debug, Clone, Serialize)]
pub struct RedeemTeamInviteResponseData {
    pub invite: Team_Invite,
}