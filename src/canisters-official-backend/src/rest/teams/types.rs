// src/rest/teams/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::{
    state::teams::types::{Team, TeamID},
    types::UserID
}, rest::{types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_url, validate_url_endpoint, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};


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
pub type ListTeamsResponse<'a> = ApiResponse<'a, ListTeamsResponseData>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTeamRequestBody {
    pub action: UpsertActionTypeEnum,
    pub name: String,
    pub avatar: Option<String>,
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
        
        // Validate avatar if provided
        if let Some(avatar) = &self.avatar {
            validate_url(avatar, "avatar")?;
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
    pub action: UpsertActionTypeEnum,
    pub id: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
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

        // Validate avatar if provided
        if let Some(avatar) = &self.avatar {
            validate_url(avatar, "avatar")?;
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


pub type GetTeamResponse<'a> = ApiResponse<'a, Team>;
pub type CreateTeamResponse<'a> = ApiResponse<'a, Team>;
pub type UpdateTeamResponse<'a> = ApiResponse<'a, Team>;
pub type DeleteTeamResponse<'a> = ApiResponse<'a, DeletedTeamData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ValidateTeamResponse<'a> = ApiResponse<'a, ValidateTeamResponseData>;