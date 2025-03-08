// src/core/state/contacts/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::core::{api::permissions::system::check_system_permissions, state::{drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, tags::types::{redact_tag, TagStringValue}, teams::types::TeamID}, types::{ICPPrincipalString, PublicKeyICP, UserID}};


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
    pub webhook_url: Option<String>, // acts as an alternative to email, separate from main webhook system
    pub public_note: String,
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
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| *user_id == *owner_id.borrow());
        let is_owned = *user_id == self.id;
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Contacts),
            PermissionGranteeID::User(user_id.clone())
        );
        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(self.id.clone().to_string()));
        let permissions = check_system_permissions(
            resource_id,
            PermissionGranteeID::User(user_id.clone())
        );
        let has_edit_permissions = permissions.contains(&SystemPermissionType::Update) || table_permissions.contains(&SystemPermissionType::Update);

        // Most sensitive
        if !is_owner {
            redacted.seed_phrase = None;

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.redeem_token = None;
                redacted.private_note = None;

                // 3rd most sensitive
                if !is_owned {
                    redacted.webhook_url = None;
                    redacted.from_placeholder_user_id = None;
                }
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