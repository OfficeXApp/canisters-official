// src/rest/teams/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::{
    state::teams::types::{Team, TeamID},
    types::{CanisterID, UserID}
}, rest::webhooks::types::SortDirection};

#[derive(Debug, Clone, Serialize)]
pub enum TeamResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> TeamResponse<'a, T> {
    pub fn ok(data: &'a T) -> TeamResponse<T> {
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
pub struct ListTeamsRequestBody {
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
pub struct ListTeamsResponseData {
    pub items: Vec<Team>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}
pub type ListTeamsResponse<'a> = TeamResponse<'a, ListTeamsResponseData>;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTeamRequestBody {
    pub name: String,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub canister_id: CanisterID,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamRequestBody {
    pub id: String,
    pub name: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertTeamRequestBody {
    Create(CreateTeamRequestBody),
    Update(UpdateTeamRequestBody),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTeamRequestBody {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedTeamData {
    pub id: String,
    pub deleted: bool
}

pub type GetTeamResponse<'a> = TeamResponse<'a, Team>;
pub type CreateTeamResponse<'a> = TeamResponse<'a, Team>;
pub type UpdateTeamResponse<'a> = TeamResponse<'a, Team>;
pub type DeleteTeamResponse<'a> = TeamResponse<'a, DeletedTeamData>;
pub type ErrorResponse<'a> = TeamResponse<'a, ()>;