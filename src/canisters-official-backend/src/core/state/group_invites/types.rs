use std::fmt;

// src/core/state/group_invites/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::api::permissions::system::check_system_permissions;
use crate::core::state::drives::state::state::OWNER_ID;
use crate::core::state::drives::types::{ExternalID, ExternalPayload};
use crate::core::state::permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum};
use crate::core::state::labels::types::{redact_label, LabelStringValue};
use crate::core::state::groups::types::GroupID;
use crate::core::types::{UserID};
use crate::rest::group_invites::types::GroupInviteFE;


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct GroupInviteID(pub String);
impl fmt::Display for GroupInviteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct GroupInvite {
    pub id: GroupInviteID,
    pub group_id: GroupID,
    pub inviter_id: UserID,
    pub invitee_id: GroupInviteeID,
    pub role: GroupRole,
    pub note: String,
    pub active_from: u64,
    pub expires_at: i64,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_invitee: Option<String>,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}


impl GroupInvite {

    pub fn cast_fe(&self, user_id: &UserID) -> GroupInviteFE {
        let group_invite = self.clone();
        // Collect group invites for this user
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Group(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Groups),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        let (group_name, group_avatar) = match crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE.with(|groups| groups.borrow().get(&group_invite.group_id).cloned()) {
            Some(group) => {
                let group_name = group.name;
                let group_avatar = group.avatar;
                (group_name, group_avatar)
            },
            None => {
                let group_name = "".to_string();
                let group_avatar = None;
                (group_name, group_avatar)
            }
        };

        let (invitee_name, invitee_avatar) = match group_invite.clone().invitee_id {
            GroupInviteeID::User(user_id) => {
                let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                    .with(|contacts| contacts.borrow().get(&user_id.clone()).cloned());
                if let Some(contact) = contact_opt {
                    (contact.name, contact.avatar)
                } else {
                    ("".to_string(), None)
                }
            },
            GroupInviteeID::PlaceholderGroupInvitee(placeholder_id) => {
                ("".to_string(), None)
            },
            GroupInviteeID::Public => {
                ("Public".to_string(), None)
            }
        };

        let invitee_id = match &self.invitee_id {
            GroupInviteeID::User(user_id) => user_id.to_string(),
            GroupInviteeID::PlaceholderGroupInvitee(placeholder_id) => placeholder_id.to_string(),
            GroupInviteeID::Public => "PUBLIC".to_string(),
        };
        
        GroupInviteFE {
            id: group_invite.id,
            group_id: group_invite.group_id,
            inviter_id: group_invite.inviter_id,
            invitee_id,
            role: group_invite.role,
            note: group_invite.note,
            active_from: group_invite.active_from,
            expires_at: group_invite.expires_at,
            created_at: group_invite.created_at,
            last_modified_at: group_invite.last_modified_at,
            from_placeholder_invitee: group_invite.from_placeholder_invitee,
            labels: group_invite.labels,
            external_id: group_invite.external_id,
            external_payload: group_invite.external_payload,
            group_name,
            group_avatar,
            invitee_name,
            invitee_avatar,
            permission_previews
        }.redacted(user_id)
    }

    
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GroupRole {
    Admin,
    Member
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct PlaceholderGroupInviteeID(pub String);
impl fmt::Display for PlaceholderGroupInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum GroupInviteeID {
    User(UserID),
    PlaceholderGroupInvitee(PlaceholderGroupInviteeID),
    Public
}
impl fmt::Display for GroupInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupInviteeID::User(user_id) => write!(f, "{}", user_id),
            GroupInviteeID::PlaceholderGroupInvitee(placeholder_id) => write!(f, "{}", placeholder_id),
            GroupInviteeID::Public => write!(f, "PUBLIC"),
        }
    }
}