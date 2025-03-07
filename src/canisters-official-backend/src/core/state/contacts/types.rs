// src/core/state/contacts/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};

use crate::core::{state::{drives::types::{ExternalID, ExternalPayload}, tags::types::TagStringValue, teams::types::TeamID}, types::{ICPPrincipalString, PublicKeyICP, UserID}};


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Contact {
    pub id: UserID,
    pub nickname: String,
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
}
