// src/rest/teams/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::{
    api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::DriveRESTUrlEndpoint}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::{state::validate_uuid4_string_with_prefix, types::redact_tag}, team_invites::types::TeamInviteID, teams::{state::state::is_team_admin, types::{Team, TeamID}}}, types::{ClientSuggestedUUID, IDPrefix, UserID}
}, rest::{types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_short_string, validate_unclaimed_uuid, validate_url, validate_url_endpoint, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamFE {
    #[serde(flatten)] // this lets us "extend" the Contact struct
    pub team: Team,
    pub member_previews: Vec<TeamMemberPreview>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl TeamFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);
        let is_team_admin = is_team_admin(user_id, &self.team.id);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions && !is_team_admin {
                redacted.team.endpoint_url = DriveRESTUrlEndpoint("".to_string());
                redacted.team.private_note = None;
            }
        }
        // Filter tags
        redacted.team.tags = match is_owner {
            true => redacted.team.tags,
            false => redacted.team.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberPreview {
    pub user_id: UserID,
    pub name: String,
    pub note: Option<String>,
    pub avatar: Option<String>,
    pub team_id: TeamID,
    pub is_admin: bool,
    pub invite_id: TeamInviteID,
    pub last_online_ms: u64,
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
    pub items: Vec<TeamFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}
pub type ListTeamsResponse<'a> = ApiResponse<'a, ListTeamsResponseData>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTeamRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub name: String,
    pub avatar: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::Team)?;
        }
        
        // Validate name (up to 256 chars)
        validate_short_string(&self.name, "name")?;

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

        // Validate endpoint_url if provided
        if let Some(endpoint_url) = &self.endpoint_url {
            validate_url_endpoint(endpoint_url, "endpoint_url")?;
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
    pub avatar: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl UpdateTeamRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate team ID
        validate_id_string(&self.id, "id")?;

        // Validate name if provided
        if let Some(name) = &self.name {
            validate_short_string(name, "name")?;
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

        // Validate endpoint_url if provided
        if let Some(endpoint_url) = &self.endpoint_url {
            validate_url_endpoint(endpoint_url, "endpoint_url")?;
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


pub type GetTeamResponse<'a> = ApiResponse<'a, TeamFE>;
pub type CreateTeamResponse<'a> = ApiResponse<'a, TeamFE>;
pub type UpdateTeamResponse<'a> = ApiResponse<'a, TeamFE>;
pub type DeleteTeamResponse<'a> = ApiResponse<'a, DeletedTeamData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ValidateTeamResponse<'a> = ApiResponse<'a, ValidateTeamResponseData>;