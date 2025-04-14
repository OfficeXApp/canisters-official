

// src/core/state/api_keys/types.rs
use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde_diff::{Diff, SerdeDiff};
use serde::{Deserialize, Serialize};
use crate::{core::{api::permissions::system::check_system_permissions, state::{contacts::state::state::CONTACTS_BY_ID_HASHTABLE, drives::{state::state::OWNER_ID, types::{ExternalID, ExternalPayload}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}, labels::types::{redact_label, LabelStringValue}}, types::UserID}, rest::api_keys::types::ApiKeyFE};
use std::{borrow::Cow, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord, CandidType)]
pub struct ApiKeyID(pub String);

impl Storable for ApiKeyID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize ApiKeyID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize ApiKeyID")
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord, CandidType)]
pub struct ApiKeyValue(pub String);

impl Storable for ApiKeyValue {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize ApiKeyValue");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize ApiKeyValue")
    }
}


// Implement Display for ApiKey
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API_KEY {{ id: {}, name: {}, user_id: {} }}", 
            self.id, self.name, self.user_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, PartialOrd, Ord, PartialEq, Eq, CandidType)]
pub struct ApiKey {
    pub id: ApiKeyID,
    pub value: ApiKeyValue,
    pub user_id: UserID,
    pub name: String,
    pub private_note: Option<String>,
    pub created_at: u64,
    pub begins_at: u64,
    pub expires_at: i64, 
    pub is_revoked: bool,
    pub labels: Vec<LabelStringValue>,
    pub external_id: Option<ExternalID>,
    pub external_payload: Option<ExternalPayload>,
}

impl Storable for ApiKey {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 64, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize ApiKeyID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize ApiKeyID")
    }
}


impl ApiKey {

    pub fn cast_fe(&self, user_id: &UserID) -> ApiKeyFE {
        let apiKey = self.clone();

        // check CONTACTS_BY_ID_HASHTABLE for contact to get its name, otherwise return "Unknown"
        let user_name = CONTACTS_BY_ID_HASHTABLE.with(|map| {
            match map.borrow().get(&apiKey.user_id) {
                Some(contact) => Some(contact.name.clone()),
                None => None
            }
        }).unwrap_or("Unknown".to_string());
        
        // Get user's system permissions for this contact record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::ApiKey(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Api_Keys),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
        .into_iter()
        .chain(table_permissions)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

        ApiKeyFE {
            apiKey,
            user_name: Some(user_name),
            permission_previews
        }.redacted(user_id)
    }

    
}


// Implement Display for ApiKeyID
impl fmt::Display for ApiKeyID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement Display for ApiKeyValue
impl fmt::Display for ApiKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, SerdeDiff)]
pub struct ApiKeyIDList {
    pub keys: Vec<ApiKeyID>,
}
impl ApiKeyIDList {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }
    
    pub fn with_key(key_id: ApiKeyID) -> Self {
        Self { keys: vec![key_id] }
    }
    
    pub fn add(&mut self, key_id: ApiKeyID) {
        self.keys.push(key_id);
    }
    
    pub fn remove(&mut self, key_id: &ApiKeyID) -> bool {
        if let Some(pos) = self.keys.iter().position(|k| k == key_id) {
            self.keys.remove(pos);
            true
        } else {
            false
        }
    }
    
    // Add iter method to satisfy the error in handler.rs
    pub fn iter(&self) -> impl Iterator<Item = &ApiKeyID> {
        self.keys.iter()
    }
    
    // Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

// Implement conversion between Vec<ApiKeyID> and ApiKeyIDList
impl From<Vec<ApiKeyID>> for ApiKeyIDList {
    fn from(keys: Vec<ApiKeyID>) -> Self {
        Self { keys }
    }
}

impl From<ApiKeyIDList> for Vec<ApiKeyID> {
    fn from(list: ApiKeyIDList) -> Self {
        list.keys
    }
}
// Implement Storable for ApiKeyIDList
impl Storable for ApiKeyIDList {
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

    
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, CandidType)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthTypeEnum {
    Signature,
    ApiKey
}
impl fmt::Display for AuthTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthTypeEnum::Signature => write!(f, "SIGNATURE"),
            AuthTypeEnum::ApiKey => write!(f, "API_KEY"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, CandidType)]
#[serde(untagged)]
pub enum AuthJsonDecoded {
    Signature(SignatureAuthProof),
    ApiKey(ApiKeyProof),
}


#[derive(Deserialize, Serialize, Debug, CandidType)]
pub struct ApiKeyProof {
    pub auth_type: AuthTypeEnum,
    pub value: ApiKeyValue,
}

#[derive(Deserialize, Serialize, Debug, CandidType)]
pub struct SignatureAuthProof {
    pub auth_type: AuthTypeEnum,
    pub challenge: SignatureAuthChallenge,
    pub signature: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, CandidType)]
pub struct SignatureAuthChallenge {
    pub timestamp_ms: u64,
    pub drive_canister_id: String,
    pub self_auth_principal: Vec<u8>,
    pub canonical_principal: String,
}