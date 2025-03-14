// src/rest/groups/types.rs
use serde::{Deserialize, Serialize};
use crate::{core::{
    api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::DriveRESTUrlEndpoint}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, labels::{state::validate_uuid4_string_with_prefix, types::redact_label}, group_invites::types::GroupInviteID, groups::{state::state::is_group_admin, types::{Group, GroupID}}}, types::{ClientSuggestedUUID, IDPrefix, UserID}
}, rest::{types::{validate_description, validate_external_id, validate_external_payload, validate_id_string, validate_short_string, validate_unclaimed_uuid, validate_url, validate_url_endpoint, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFE {
    #[serde(flatten)] // this lets us "extend" the Contact struct
    pub group: Group,
    pub member_previews: Vec<GroupMemberPreview>,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl GroupFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);
        let is_group_admin = is_group_admin(user_id, &self.group.id);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions && !is_group_admin {
                redacted.group.endpoint_url = DriveRESTUrlEndpoint("".to_string());
                redacted.group.private_note = None;
            }
        }
        // Filter labels
        redacted.group.labels = match is_owner {
            true => redacted.group.labels,
            false => redacted.group.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberPreview {
    pub user_id: UserID,
    pub name: String,
    pub note: Option<String>,
    pub avatar: Option<String>,
    pub group_id: GroupID,
    pub is_admin: bool,
    pub invite_id: GroupInviteID,
    pub last_online_ms: u64,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ListGroupsRequestBody {
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

impl ListGroupsRequestBody {
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
pub struct ListGroupsResponseData {
    pub items: Vec<GroupFE>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}
pub type ListGroupsResponse<'a> = ApiResponse<'a, ListGroupsResponseData>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGroupRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub name: String,
    pub avatar: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl CreateGroupRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::Group)?;
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
pub struct UpdateGroupRequestBody {
    pub id: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub endpoint_url: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}
impl UpdateGroupRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate group ID
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
pub struct DeleteGroupRequestBody {
    pub id: String,
}
impl DeleteGroupRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate group ID
        validate_id_string(&self.id, "id")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedGroupData {
    pub id: String,
    pub deleted: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateGroupRequestBody {
    pub user_id: UserID, // does this user
    pub group_id: GroupID, // belong to this group
    // pub signature: String, // relay the signature to the cosmic drive to prove user_id
}
impl ValidateGroupRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate user_id
        validate_user_id(&self.user_id.0)?;
        
        // Validate group_id
        validate_id_string(&self.group_id.0, "group_id")?;
        
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateGroupResponseData {
    pub is_member: bool,
    pub group_id: GroupID,
    pub user_id: UserID
}


pub type GetGroupResponse<'a> = ApiResponse<'a, GroupFE>;
pub type CreateGroupResponse<'a> = ApiResponse<'a, GroupFE>;
pub type UpdateGroupResponse<'a> = ApiResponse<'a, GroupFE>;
pub type DeleteGroupResponse<'a> = ApiResponse<'a, DeletedGroupData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ValidateGroupResponse<'a> = ApiResponse<'a, ValidateGroupResponseData>;