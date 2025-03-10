// src/core/state/teams/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;
use crate::{core::{
    api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{DriveID, DriveRESTUrlEndpoint, ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}, team_invites::types::{TeamInviteID, TeamInviteeID}}, types::UserID
}, rest::teams::types::{TeamFE, TeamMemberPreview}};
use serde_diff::{SerdeDiff};
use std::iter::Iterator;
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
    pub endpoint_url: DriveRESTUrlEndpoint,
    pub tags: Vec<TagStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Team {

    pub fn cast_fe(&self, user_id: &UserID) -> TeamFE {
        let team = self.clone();
        // Collect team invites for this user
        let mut member_previews = Vec::new();
        
        for invite_id in &team.member_invites {
            // Get the invite data
            let invite_opt = crate::core::state::team_invites::state::state::INVITES_BY_ID_HASHTABLE
                .with(|invites| invites.borrow().get(invite_id).cloned());
            
            if let Some(invite) = invite_opt {
                // Check if user is an admin
                let is_admin = is_team_admin(&user_id.clone(), &team.id);

                // query the contacts hashtable by invitee_id to get the name and avatar
                // we have to check that TeamInviteeID::User matches
                let invitee_name = match invite.invitee_id.clone() {
                    TeamInviteeID::User(user_id) => {
                        // query the contacts hashtable by user_id to get the name
                        let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                            .with(|contacts| contacts.borrow().get(&user_id.clone()).cloned());
                        if let Some(contact) = contact_opt {
                            contact.name
                        } else {
                            "".to_string()
                        }
                    },
                    _ => "".to_string()
                };
                let invitee_avatar = match invite.invitee_id.clone() {
                    TeamInviteeID::User(user_id) => {
                        // query the contacts hashtable by user_id to get the avatar
                        let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                            .with(|contacts| contacts.borrow().get(&user_id.clone()).cloned());
                        if let Some(contact) = contact_opt {
                            contact.avatar
                        } else {
                            None
                        }
                    },
                    _ => None
                };
                
                member_previews.push(TeamMemberPreview {
                    user_id: UserID(invite.invitee_id.to_string()),
                    name: invitee_name,
                    avatar: invitee_avatar,
                    is_admin,
                    team_id: team.id.clone(),
                    invite_id: invite.id.clone(),
                });
            }
        }
        
        
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

        TeamFE {
            team,
            member_previews,
            permission_previews
        }.redacted(user_id)
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