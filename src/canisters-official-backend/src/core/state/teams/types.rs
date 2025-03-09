// src/core/state/teams/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;
use crate::core::{
    api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{DriveID, DriveRESTUrlEndpoint, ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}, team_invites::types::TeamInviteID}, types::UserID
};
use serde_diff::{SerdeDiff};

use super::state::state::is_team_admin;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TeamID(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Team {
    pub id: TeamID,
    pub name: String,
    pub owner: UserID,
    pub avatar: Option<String>,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
    pub admin_invites: Vec<TeamInviteID>, // all admin_invites are also in member_invites
    pub member_invites: Vec<TeamInviteID>,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub drive_id: DriveID,
    pub url_endpoint: DriveRESTUrlEndpoint,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}
impl Team {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Teams),
            PermissionGranteeID::User(user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        let permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(user_id.clone())
        );
        let has_edit_permissions = permissions.contains(&SystemPermissionType::Edit) || table_permissions.contains(&SystemPermissionType::Edit);
        let is_team_admin = is_team_admin(user_id, &self.id);

        // Most sensitive
        if !is_owner {

            // 2nd most sensitive
            if !has_edit_permissions && !is_team_admin {
                redacted.admin_invites = vec![];
                redacted.member_invites = vec![];
                redacted.private_note = None;
            }
        }
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

// Implement Display for TeamID
impl fmt::Display for TeamID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement Display for Team
impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Team {{ id: {}, name: {}, owner: {} }}", 
            self.id, self.name, self.owner)
    }
}