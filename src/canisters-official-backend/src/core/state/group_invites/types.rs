use std::borrow::Cow;
use std::fmt;

use candid::CandidType;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
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


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct GroupInviteID(pub String);
impl fmt::Display for GroupInviteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for GroupInviteID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GroupInviteID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GroupInviteID")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord, PartialEq, Eq)]
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
    pub redeem_code: Option<String>,
    pub from_placeholder_invitee: Option<String>,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Storable for GroupInvite {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize GroupInvite");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize GroupInvite")
    }
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

        let (group_name, group_avatar) = match crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE.with(|groups| groups.borrow().get(&group_invite.group_id).clone()) {
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
                    .with(|contacts| contacts.borrow().get(&user_id.clone()));
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
            redeem_code: group_invite.redeem_code,
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


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GroupRole {
    Admin,
    Member
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct PlaceholderGroupInviteeID(pub String);
impl fmt::Display for PlaceholderGroupInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Storable for PlaceholderGroupInviteeID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize PlaceholderGroupInviteeID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize PlaceholderGroupInviteeID")
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub enum GroupInviteeID {
    User(UserID),
    PlaceholderGroupInvitee(PlaceholderGroupInviteeID),
    Public
}

impl Storable for GroupInviteeID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize ICPPrincipalString");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize ICPPrincipalString")
    }
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


#[derive(Clone, Debug, CandidType, Deserialize, Serialize, SerdeDiff)]
#[derive(Default)]
pub struct GroupInviteIDList {
    pub invites: Vec<GroupInviteID>,
}

impl GroupInviteIDList {
    pub fn new() -> Self {
        Self { invites: Vec::new() }
    }
    
    pub fn with_invite(invite_id: GroupInviteID) -> Self {
        Self { invites: vec![invite_id] }
    }
    
    pub fn add(&mut self, invite_id: GroupInviteID) {
        self.invites.push(invite_id);
    }
    
    pub fn remove(&mut self, invite_id: &GroupInviteID) -> bool {
        if let Some(pos) = self.invites.iter().position(|i| i == invite_id) {
            self.invites.remove(pos);
            true
        } else {
            false
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &GroupInviteID> {
        self.invites.iter()
    }
    
    pub fn is_empty(&self) -> bool {
        self.invites.is_empty()
    }
    
    pub fn contains(&self, invite_id: &GroupInviteID) -> bool {
        self.invites.contains(invite_id)
    }
    
    pub fn len(&self) -> usize {
        self.invites.len()
    }
    
    pub fn get(&self, index: usize) -> Option<&GroupInviteID> {
        self.invites.get(index)
    }
}

// Implement conversion between Vec<GroupInviteID> and GroupInviteIDList
impl From<Vec<GroupInviteID>> for GroupInviteIDList {
    fn from(invites: Vec<GroupInviteID>) -> Self {
        Self { invites }
    }
}

impl From<GroupInviteIDList> for Vec<GroupInviteID> {
    fn from(list: GroupInviteIDList) -> Self {
        list.invites
    }
}

// Implement Storable for GroupInviteIDList
impl Storable for GroupInviteIDList {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 1024 * 4, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}