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
use crate::rest::team_invites::types::TeamInviteFE;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TeamInviteID(pub String);
impl fmt::Display for TeamInviteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct TeamInvite {
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
    pub from_placeholder_invitee: Option<String>,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}


impl TeamInvite {

    pub fn cast_fe(&self, user_id: &UserID) -> TeamInviteFE {
        let team_invite = self.clone();
        // Collect team invites for this user
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Team(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Teams),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        let (team_name, team_avatar) = match crate::core::state::teams::state::state::TEAMS_BY_ID_HASHTABLE.with(|teams| teams.borrow().get(&team_invite.team_id).cloned()) {
            Some(team) => {
                let team_name = team.name;
                let team_avatar = team.avatar;
                (team_name, team_avatar)
            },
            None => {
                let team_name = "".to_string();
                let team_avatar = None;
                (team_name, team_avatar)
            }
        };

        let (invitee_name, invitee_avatar) = match team_invite.clone().invitee_id {
            TeamInviteeID::User(user_id) => {
                let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                    .with(|contacts| contacts.borrow().get(&user_id.clone()).cloned());
                if let Some(contact) = contact_opt {
                    (contact.name, contact.avatar)
                } else {
                    ("".to_string(), None)
                }
            },
            TeamInviteeID::PlaceholderTeamInvitee(placeholder_id) => {
                ("".to_string(), None)
            },
            TeamInviteeID::Public => {
                ("Public".to_string(), None)
            }
        };

        let invitee_id = match &self.invitee_id {
            TeamInviteeID::User(user_id) => user_id.to_string(),
            TeamInviteeID::PlaceholderTeamInvitee(placeholder_id) => placeholder_id.to_string(),
            TeamInviteeID::Public => "PUBLIC".to_string(),
        };

        TeamInviteFE {
            id: team_invite.id,
            team_id: team_invite.team_id,
            inviter_id: team_invite.inviter_id,
            invitee_id,
            role: team_invite.role,
            note: team_invite.note,
            active_from: team_invite.active_from,
            expires_at: team_invite.expires_at,
            created_at: team_invite.created_at,
            last_modified_at: team_invite.last_modified_at,
            from_placeholder_invitee: team_invite.from_placeholder_invitee,
            tags: team_invite.tags,
            external_id: team_invite.external_id,
            external_payload: team_invite.external_payload,
            team_name,
            team_avatar,
            invitee_name,
            invitee_avatar,
            permission_previews
        }.redacted(user_id)
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
    Public
}
impl fmt::Display for TeamInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TeamInviteeID::User(user_id) => write!(f, "{}", user_id),
            TeamInviteeID::PlaceholderTeamInvitee(placeholder_id) => write!(f, "{}", placeholder_id),
            TeamInviteeID::Public => write!(f, "PUBLIC"),
        }
    }
}