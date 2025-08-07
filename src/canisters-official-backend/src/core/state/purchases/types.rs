use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;
use std::{borrow::Cow, fmt};

use crate::{
    core::{
        api::permissions::system::check_system_permissions,
        state::{
            permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum},
        },
        types::UserID,
    },
    rest::purchases::types::PurchaseFE,
};

/// Represents a unique identifier for a Purchase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, PartialOrd, Ord)]
pub struct PurchaseID(pub String);

impl fmt::Display for PurchaseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for PurchaseID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize PurchaseID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize PurchaseID")
    }
}



/// Defines the possible statuses for a Purchase.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, SerdeDiff, CandidType)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurchaseStatus {
    Requested,
    Awaiting,
    Running,
    Blocked,
    Completed,
    Failed,
    Canceled,
    PaymentRequired,
    Refunded,
    Archived,
    Unknown,
}

impl fmt::Display for PurchaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PurchaseStatus::Requested => write!(f, "REQUESTED"),
            PurchaseStatus::Awaiting => write!(f, "AWAITING"),
            PurchaseStatus::Running => write!(f, "RUNNING"),
            PurchaseStatus::Blocked => write!(f, "BLOCKED"),
            PurchaseStatus::Completed => write!(f, "COMPLETED"),
            PurchaseStatus::Failed => write!(f, "FAILED"),
            PurchaseStatus::Canceled => write!(f, "CANCELED"),
            PurchaseStatus::PaymentRequired => write!(f, "PAYMENT_REQUIRED"),
            PurchaseStatus::Refunded => write!(f, "REFUNDED"),
            PurchaseStatus::Archived => write!(f, "ARCHIVED"),
            PurchaseStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, SerdeDiff)]
pub struct PurchaseIDList {
    pub purchases: Vec<PurchaseID>,
}

/// Represents the full details of a Purchase record.
#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub struct Purchase {
    pub id: PurchaseID,
    pub template_id: Option<String>, // no guarantees on this, only set on create
    pub vendor_name: String, // cannot be updated, only set on create
    pub vendor_id: UserID, // cannot be updated, only set on create
    pub status: PurchaseStatus, // can be updated by vendor
    pub description: String, // cannot be updated, only set on create
    pub about_url: String,
    pub billing_url: String, // can be updated by vendor
    pub support_url: String, // can be updated by vendor
    pub delivery_url: String, // can be updated by vendor
    pub verification_url: String, // can be updated by vendor
    pub auth_installation_url: String, // the script to run to install the purchase
    pub title: String, // cannot be updated, only set on create
    pub subtitle: String, // can be updated
    pub pricing: String, // can be updated
    pub next_delivery_date: i64, // can be updated by vendor
    pub vendor_notes: String, // can be updated by vendor
    pub notes: String, // cannot be viewed or updated by vendor
    pub created_at: u64,
    pub updated_at: u64,
    pub last_updated_at: u64,
    pub related_resources: Vec<String>, // list of ID strings, can be updated
    pub tracer: Option<String>, // can be updated by vendor
    pub labels: Vec<String>, // can be updated by vendor
    pub external_id: Option<String>, // can be updated by vendor
    pub external_payload: Option<String>, // can be updated by vendor
}

impl Storable for Purchase {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256 * 256, // Adjust based on your needs for a larger struct
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize Purchase");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize Purchase")
    }
}

impl Purchase {
    /// Casts the internal Purchase representation to its frontend equivalent,
    /// applying permission-based redactions.
    pub fn cast_fe(&self, user_id: &UserID) -> PurchaseFE {
        let purchase = self.clone();

        // Get user's system permissions for this purchase record
        let record_permissions = check_system_permissions(
            SystemResourceID::Record(SystemRecordIDEnum::Purchase(self.id.to_string())),
            PermissionGranteeID::User(user_id.clone())
        );
        let table_permissions = check_system_permissions(
            SystemResourceID::Table(SystemTableEnum::Purchases),
            PermissionGranteeID::User(user_id.clone())
        );
        let permission_previews: Vec<SystemPermissionType> = record_permissions
            .into_iter()
            .chain(table_permissions)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        PurchaseFE {
            purchase,
            permission_previews
        }.redacted(user_id)
    }
}