// src/rest/teams/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::{
    state::teams::types::{Team, TeamID},
    types::UserID
}, rest::webhooks::types::SortDirection, types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_url_endpoint, validate_user_id, ValidationError}};

#[derive(Debug, Clone, Serialize)]
pub enum TeamResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> TeamResponse<'a, T> {
    pub fn ok(data: &'a T) -> TeamResponse<'a, T> {
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

impl ListTeamsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
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
pub struct ListTeamsResponseData {
    pub items: Vec<Team>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}
pub type ListTeamsResponse<'a> = TeamResponse<'a, ListTeamsResponseData>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTeamRequestBody {
    pub name: String,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub url_endpoint: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate name (up to 256 chars)
        validate_id_string(&self.name, "name")?;

        // Validate public_note if provided
        if let Some(public_note) = &self.public_note {
            validate_description(public_note, "public_note")?;
        }

        // Validate private_note if provided
        if let Some(private_note) = &self.private_note {
            validate_description(private_note, "private_note")?;
        }

        // Validate url_endpoint if provided
        if let Some(url_endpoint) = &self.url_endpoint {
            validate_url_endpoint(url_endpoint, "url_endpoint")?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamRequestBody {
    pub id: String,
    pub name: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub url_endpoint: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl UpdateTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate team ID
        validate_id_string(&self.id, "id")?;

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_id_string(name, "name")?;
        }

        // Validate public_note if provided
        if let Some(public_note) = &self.public_note {
            validate_description(public_note, "public_note")?;
        }

        // Validate private_note if provided
        if let Some(private_note) = &self.private_note {
            validate_description(private_note, "private_note")?;
        }

        // Validate url_endpoint if provided
        if let Some(url_endpoint) = &self.url_endpoint {
            validate_url_endpoint(url_endpoint, "url_endpoint")?;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpsertTeamRequestBody {
    Create(CreateTeamRequestBody),
    Update(UpdateTeamRequestBody),
}
impl UpsertTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertTeamRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertTeamRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTeamRequestBody {
    pub id: String,
}
impl DeleteTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate team ID
        validate_id_string(&self.id, "id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedTeamData {
    pub id: String,
    pub deleted: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTeamRequestBody {
    pub user_id: UserID, // does this user
    pub team_id: TeamID, // belong to this team
    // pub signature: String, // relay the signature to the cosmic drive to prove user_id
}
impl ValidateTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate user_id
        validate_user_id(&self.user_id.0)?;
        
        // Validate team_id
        validate_id_string(&self.team_id.0, "team_id")?;
        
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTeamResponseData {
    pub is_member: bool,
    pub team_id: TeamID,
    pub user_id: UserID
}


pub type GetTeamResponse<'a> = TeamResponse<'a, Team>;
pub type CreateTeamResponse<'a> = TeamResponse<'a, Team>;
pub type UpdateTeamResponse<'a> = TeamResponse<'a, Team>;
pub type DeleteTeamResponse<'a> = TeamResponse<'a, DeletedTeamData>;
pub type ErrorResponse<'a> = TeamResponse<'a, ()>;
pub type ValidateTeamResponse<'a> = TeamResponse<'a, ValidateTeamResponseData>;