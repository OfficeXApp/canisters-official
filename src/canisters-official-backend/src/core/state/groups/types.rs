use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
// src/core/state/groups/types.rs
use serde::{Serialize, Deserialize};
use std::{borrow::Cow, fmt};
use crate::{core::{
    api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{DriveID, DriveRESTUrlEndpoint, ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, labels::types::{redact_label, LabelStringValue}, group_invites::types::{GroupInviteID, GroupInviteeID, GroupRole}}, types::UserID
}, rest::groups::types::{GroupFE, GroupMemberPreview}};
use serde_diff::{SerdeDiff};
use std::iter::Iterator;
use super::state::state::is_group_admin;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct GroupID(pub String);

impl Storable for GroupID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GroupID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GroupID")
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd, PartialEq, Eq)]
pub struct Group {
    pub id: GroupID,
    pub name: String,
    pub owner: UserID,
    pub avatar: Option<String>,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
    pub admin_invites: Vec<GroupInviteID>, // all admin_invites are also in member_invites
    pub member_invites: Vec<GroupInviteID>,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub drive_id: DriveID,
    pub host_url: DriveRESTUrlEndpoint,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Storable for Group {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 1024, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize Group");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize Group")
    }
}

impl Group {

    pub fn cast_fe(&self, user_id: &UserID) -> GroupFE {
        let group = self.clone();
        // Collect group invites for this user
        let mut member_previews = Vec::new();
        
        for invite_id in &group.member_invites {
            // Get the invite data
            let invite_opt = crate::core::state::group_invites::state::state::INVITES_BY_ID_HASHTABLE
                .with(|invites| invites.borrow().get(invite_id).clone());
            
            if let Some(invite) = invite_opt {
                

                // query the contacts hashtable by invitee_id to get the name and avatar
                // we have to check that GroupInviteeID::User matches
                let invitee_name = match invite.invitee_id.clone() {
                    GroupInviteeID::User(user_id) => {
                        // query the contacts hashtable by user_id to get the name
                        let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                            .with(|contacts| contacts.borrow().get(&user_id.clone()).map(|data| data.clone()));
                        if let Some(contact) = contact_opt {
                            contact.name
                        } else {
                            "".to_string()
                        }
                    },
                    _ => "".to_string()
                };
                let invitee_avatar = match invite.invitee_id.clone() {
                    GroupInviteeID::User(user_id) => {
                        // query the contacts hashtable by user_id to get the avatar
                        let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                            .with(|contacts| contacts.borrow().get(&user_id.clone()).map(|data| data.clone()));
                        if let Some(contact) = contact_opt {
                            contact.avatar
                        } else {
                            None
                        }
                    },
                    _ => None
                };
                let invitee_last_online_ms = match invite.invitee_id.clone() {
                    GroupInviteeID::User(user_id) => {
                        // query the contacts hashtable by user_id to get the last_active
                        let contact_opt = crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE
                            .with(|contacts| contacts.borrow().get(&user_id.clone()).map(|data| data.clone()));
                        if let Some(contact) = contact_opt {
                            contact.last_online_ms
                        } else {
                            0
                        }
                    },
                    _ => 0
                };
                let is_admin = match invite.role {
                    GroupRole::Admin => true,
                    _ => false
                };
                
                member_previews.push(GroupMemberPreview {
                    user_id: UserID(invite.invitee_id.to_string()),
                    name: invitee_name,
                    avatar: invitee_avatar,
                    note: Some(invite.note),
                    last_online_ms: invitee_last_online_ms,
                    is_admin,
                    group_id: group.id.clone(),
                    invite_id: invite.id.clone(),
                });
            }
        }
        
        
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

        GroupFE {
            group,
            member_previews,
            permission_previews
        }.redacted(user_id)
    }

    
}


// Implement Display for GroupID
impl fmt::Display for GroupID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement Display for Group
impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Group {{ id: {}, name: {}, owner: {} }}", 
            self.id, self.name, self.owner)
    }
}