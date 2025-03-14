// src/core/state/contacts/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::{core::{api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}, group_invites::types::GroupInviteeID, groups::types::GroupID}, types::{ICPPrincipalString, PublicKeyICP, UserID}}, rest::contacts::types::{ContactFE, ContactGroupInvitePreview}};


// frontend ui
// row colums: avatar, name, icp, last_online_ms
// popover: pub/priv note, email, evm/icp, tags
// filters: search by name/icp/email, filter by tags, groups, sort by last_online_ms, created_at

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
    pub seed_phrase: Option<String>, // careful! if we use superswap or redeem_code to change user_id, the seed_phrase wont be updated! you'll need to manually update it via UpdateContactRequestBody and obey the validation logic
    pub groups: Vec<GroupID>,
    pub tags: Vec<TagStringValue>,
    pub past_user_ids: Vec<UserID>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
    pub from_placeholder_user_id: Option<UserID>,
    pub redeem_code: Option<String>,
    pub created_at: u64,
    pub last_online_ms: u64,
}

impl Contact {

    pub fn cast_fe(&self, user_id: &UserID) -> ContactFE {
        let contact = self.clone();
        // Collect group invites for this user
        let group_previews: Vec<ContactGroupInvitePreview> = contact.groups.iter()
            .filter_map(|group_id| {
                // Get the group data
                let group_opt = crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE
                    .with(|groups| groups.borrow().get(group_id).cloned());
                
                if let Some(group) = group_opt {
                    // Find user's invite in this group
                    let invite_id_opt = group.member_invites.iter()
                        .find(|invite_id| {
                            crate::core::state::group_invites::state::state::INVITES_BY_ID_HASHTABLE
                                .with(|invites| {
                                    if let Some(invite) = invites.borrow().get(invite_id) {
                                        invite.invitee_id == GroupInviteeID::User(self.id.clone())
                                    } else {
                                        false
                                    }
                                })
                        }).cloned();
                    
                    if let Some(invite_id) = invite_id_opt {
                        // Check if user is an admin
                        let is_admin = crate::core::state::groups::state::state::is_group_admin(&self.id, group_id);
                        
                        Some(ContactGroupInvitePreview {
                            group_id: group_id.clone(),
                            invite_id,
                            is_admin,
                            group_name: group.name,
                            group_avatar: group.avatar,
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
            group_previews,
            permission_previews
        }.redacted(user_id)
    }

    
}