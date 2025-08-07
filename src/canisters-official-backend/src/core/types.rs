
// src/core/state/types.rs
use std::{borrow::Cow, fmt};
use candid::CandidType;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use serde_diff::{Diff, SerdeDiff};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct PublicKeyICP(pub String);

impl fmt::Display for PublicKeyICP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for PublicKeyICP {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize PublicKeyICP");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize PublicKeyICP")
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType, Ord, PartialOrd)]
pub struct ICPPrincipalString(pub PublicKeyICP);

impl fmt::Display for ICPPrincipalString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for ICPPrincipalString {
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



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct PublicKeyEVM(pub String);

impl fmt::Display for PublicKeyEVM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct ClientSuggestedUUID(pub String);

impl fmt::Display for ClientSuggestedUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct UserID(pub String);


impl Storable for UserID {
    const BOUND: Bound = Bound::Bounded {
        max_size: 256, // Adjust based on your needs
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes)
            .expect("Failed to serialize UserID");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref())
            .expect("Failed to deserialize UserID")
    }
}

impl fmt::Display for UserID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl UserID {
    pub fn to_icp_principal_string(&self) -> ICPPrincipalString {
        // Remove the IDPrefix::User prefix ("UserID_") if present
        let prefix = IDPrefix::User.as_str();
        if self.0.starts_with(prefix) {
            ICPPrincipalString(PublicKeyICP(self.0[prefix.len()..].to_string()))
        } else {
            // this should probably throw an error instead of just returning the same thing
            ICPPrincipalString(PublicKeyICP(self.0.clone()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub enum IDPrefix {
    File,
    Folder,
    Drive,
    FileVersion,
    ApiKey,
    Disk,
    Group,
    GroupInvite,
    Webhook,
    User,
    SystemPermission, 
    DirectoryPermission,
    PlaceholderPermissionGrantee,
    DirectoryActionOutcome,
    PlaceholderGroupInviteeID,
    ShareTrackID,
    DriveStateDiffID,
    LabelID,
    RedeemCode,
    InboxNotifID,
    Purchase,
}

impl IDPrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            IDPrefix::File => "FileID_",
            IDPrefix::Folder => "FolderID_",
            IDPrefix::FileVersion => "FileVersionID_",
            IDPrefix::Drive => "DriveID_",
            IDPrefix::ApiKey => "ApiKeyID_",
            IDPrefix::Disk => "DiskID_",
            IDPrefix::Group => "GroupID_",
            IDPrefix::GroupInvite => "GroupInviteID_",
            IDPrefix::SystemPermission => "SystemPermissionID_",
            IDPrefix::DirectoryPermission => "DirectoryPermissionID_",
            IDPrefix::PlaceholderPermissionGrantee => "PlaceholderPermissionGranteeID_",
            IDPrefix::Webhook => "WebhookID_",
            IDPrefix::User => "UserID_",
            IDPrefix::DirectoryActionOutcome => "DirectoryActionOutcomeID_",
            IDPrefix::PlaceholderGroupInviteeID => "PlaceholderGroupInviteeID_",
            IDPrefix::ShareTrackID => "ShareTrackID_",
            IDPrefix::DriveStateDiffID => "DriveStateDiffID_",
            IDPrefix::LabelID => "LabelID_",
            IDPrefix::RedeemCode => "RedeemTokenID_",
            IDPrefix::InboxNotifID => "InboxNotifID_",
            IDPrefix::Purchase => "PurchaseID_",
        }
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub enum AuthPrefixEnum {
    ApiKey,
    Signature,
}
impl AuthPrefixEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthPrefixEnum::ApiKey => "ApiKey_",
            AuthPrefixEnum::Signature => "Signature_",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff, CandidType)]
pub struct ParsedAuth {
    pub auth_type: AuthPrefixEnum,
    pub value: String,
}

pub fn parse_auth_header_value(auth_header_value: &str) -> Result<ParsedAuth, &'static str> {
    // First, check if it starts with "Bearer "
    let without_bearer = auth_header_value.strip_prefix("Bearer ")
        .ok_or("Authentication header must start with 'Bearer '")?;
    
    // Try to match against both prefix types
    if without_bearer.starts_with(AuthPrefixEnum::ApiKey.as_str()) {
        Ok(ParsedAuth {
            auth_type: AuthPrefixEnum::ApiKey,
            value: without_bearer[AuthPrefixEnum::ApiKey.as_str().len()..].to_string(),
        })
    } else if without_bearer.starts_with(AuthPrefixEnum::Signature.as_str()) {
        Ok(ParsedAuth {
            auth_type: AuthPrefixEnum::Signature,
            value: without_bearer[AuthPrefixEnum::Signature.as_str().len()..].to_string(),
        })
    } else {
        Err("Invalid authentication type prefix")
    }
}
