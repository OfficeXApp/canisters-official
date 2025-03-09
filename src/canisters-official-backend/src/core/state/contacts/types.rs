// src/core/state/contacts/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::{core::{api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}, team_invites::types::TeamInviteeID, teams::types::TeamID}, types::{ICPPrincipalString, PublicKeyICP, UserID}}, rest::contacts::types::{ContactFE, ContactTeamInvitePreview}};


// frontend ui
// row colums: avatar, name, icp, last_online_ms
// popover: pub/priv note, email, evm/icp, tags
// filters: search by name/icp/email, filter by tags, teams, sort by last_online_ms, created_at

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Contact {
    pub id: UserID,
    pub name: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub notifications_url: Option<String>, // acts as an alternative to email, separate from main webhook system
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub evm_public_address: String,
    pub icp_principal: ICPPrincipalString,
    pub seed_phrase: Option<String>, // careful! if we use superswap or redeem_token to change user_id, the seed_phrase wont be updated! you'll need to manually update it via UpdateContactRequestBody and obey the validation logic
    pub teams: Vec<TeamID>,
    pub tags: Vec<TagStringValue>,
    pub past_user_ids: Vec<UserID>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
    pub from_placeholder_user_id: Option<UserID>,
    pub redeem_token: Option<String>,
    pub created_at: u64,
    pub last_online_ms: u64,
}

impl Contact {

    pub fn cast_fe(&self, user_id: &UserID) -> ContactFE {
        let contact = self.clone();
        // Collect team invites for this user
        let team_previews: Vec<ContactTeamInvitePreview> = contact.teams.iter()
            .filter_map(|team_id| {
                // Get the team data
                let team_opt = crate::core::state::teams::state::state::TEAMS_BY_ID_HASHTABLE
                    .with(|teams| teams.borrow().get(team_id).cloned());
                
                if let Some(team) = team_opt {
                    // Find user's invite in this team
                    let invite_id_opt = team.member_invites.iter()
                        .find(|invite_id| {
                            crate::core::state::team_invites::state::state::INVITES_BY_ID_HASHTABLE
                                .with(|invites| {
                                    if let Some(invite) = invites.borrow().get(invite_id) {
                                        invite.invitee_id == TeamInviteeID::User(self.id.clone())
                                    } else {
                                        false
                                    }
                                })
                        }).cloned();
                    
                    if let Some(invite_id) = invite_id_opt {
                        // Check if user is an admin
                        let is_admin = crate::core::state::teams::state::state::is_team_admin(&self.id, team_id);
                        
                        Some(ContactTeamInvitePreview {
                            team_id: team_id.clone(),
                            invite_id,
                            is_admin,
                            team_name: team.name,
                            team_avatar: team.avatar,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::User(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Contacts),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        ContactFE {
            contact,
            team_previews,
            permission_previews
        }.redacted(user_id)
    }

    
}