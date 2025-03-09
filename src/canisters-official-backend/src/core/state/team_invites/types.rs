use std::fmt;

// src/core/state/team_invites/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::api::permissions::system::check_system_permissions;
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::drives::types::{ExternalID, ExternalPayload};
use crate::core::state::permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum};
use crate::core::state::tags::types::{redact_tag, TagStringValue};
use crate::core::state::teams::types::TeamID;
use crate::core::types::{UserID};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TeamInviteID(pub String);
impl fmt::Display for TeamInviteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Team_Invite {
    pub id: TeamInviteID,
    pub team_id: TeamID,
    pub inviter_id: UserID,
    pub invitee_id: TeamInviteeID,
    pub role: TeamRole,
    pub note: String,
    pub active_from: u64,
    pub expires_at: i64,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_invitee: Option<PlaceholderTeamInviteeID>,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Team_Invite {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        // let is_owned = self.inviter_id == *user_id || self.invitee_id == TeamInviteeID::User(user_id.clone());
        // let table_permissions = check_system_permissions(
        //     SystemResourceID::Table(SystemTableEnum::Teams),
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        // let permissions = check_system_permissions(
        //     resource_id,
        //     PermissionGranteeID::User(user_id.clone())
        // );
        // let has_edit_permissions = permissions.contains(&SystemPermissionType::Edit) || table_permissions.contains(&SystemPermissionType::Edit);

        // Filter tags
        redacted.tags = match is_owner {
            true => redacted.tags,
            false => redacted.tags.iter()
            .filter_map(|tag| redact_tag(tag.clone(), user_id.clone()))
            .collect()
        };
        
        redacted
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TeamRole {
    Admin,
    Member
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct PlaceholderTeamInviteeID(pub String);
impl fmt::Display for PlaceholderTeamInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum TeamInviteeID {
    User(UserID),
    PlaceholderTeamInvitee(PlaceholderTeamInviteeID),
}
impl fmt::Display for TeamInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TeamInviteeID::User(user_id) => write!(f, "{}", user_id),
            TeamInviteeID::PlaceholderTeamInvitee(placeholder_id) => write!(f, "{}", placeholder_id),
        }
    }
}