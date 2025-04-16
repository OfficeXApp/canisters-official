// src/rest/contacts/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::{api::permissions::system::check_system_permissions, state::{api_keys::types::ApiKeyValue, contacts::types::Contact, drives::state::state::OWNER_ID, group_invites::types::{GroupInviteID, GroupInviteeID}, groups::types::GroupID, labels::{state::validate_uuid4_string_with_prefix, types::{redact_group_previews, redact_label}}, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}}, types::{ClientSuggestedUUID, IDPrefix, UserID}}, rest::{auth::seed_phrase_to_wallet_addresses, types::{validate_email, validate_evm_address, validate_external_id, validate_external_payload, validate_id_string, validate_unclaimed_uuid, validate_url, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactFE {
    #[serde(flatten)] // this lets us "extend" the Contact struct
    pub contact: Contact,
    pub group_previews: Vec<ContactGroupInvitePreview>,
    pub permission_previews: Vec<SystemPermissionType>,
}
impl ContactFE {
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        let is_owner = OWNER_ID.with(|owner_id| user_id.clone() == owner_id.borrow().get().clone());
        let is_owned = *user_id == self.contact.id;
        let has_edit_permissions = redacted.permission_previews.contains(&SystemPermissionType::Edit);

        // Most sensitive
        if !is_owner {
            redacted.contact.seed_phrase = None;

            // 2nd most sensitive
            if !has_edit_permissions {
                redacted.contact.redeem_code = None;
                redacted.contact.private_note = None;

                // 3rd most sensitive
                if !is_owned {
                    redacted.contact.notifications_url = None;
                    redacted.contact.from_placeholder_user_id = None;
                }
            }
        }
        // Filter labels
        redacted.contact.labels = match is_owner {
            true => redacted.contact.labels,
            false => redacted.contact.labels.iter()
            .filter_map(|label| redact_label(label.clone(), user_id.clone()))
            .collect()
        };
        // Filter group previews
        let redacted_group_previews: Vec<ContactGroupInvitePreview> = redacted.group_previews.iter()
            .filter_map(|group_preview| redact_group_previews(group_preview.clone(), user_id.clone()))
            .collect();
        redacted.group_previews = redacted_group_previews;
        // this code is kinda redundant, but it's here for clarity
        redacted.permission_previews = redacted.permission_previews;
            
        redacted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroupInvitePreview {
    pub group_id: GroupID,
    pub invite_id: GroupInviteID,
    pub is_admin: bool,
    pub group_name: String,
    pub group_avatar: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ListContactsRequestBody {
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

fn default_page_size() -> usize {
    50
}

impl ListContactsRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate filters string length
        if self.filters.len() > 256 {
            return Err(ValidationError {
                field: "filters".to_string(),
                message: "Filters must be 256 characters or less".to_string(),
            });
        }

        // Validate page_size is reasonable
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(ValidationError {
                field: "page_size".to_string(),
                message: "Page size must be between 1 and 1000".to_string(),
            });
        }

        // Validate cursor strings if present
        if let Some(cursor) = &self.cursor {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }


        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct ListContactsResponseData {
    pub items: Vec<ContactFE>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

pub type GetContactResponse<'a> = ApiResponse<'a, ContactFE>;

pub type ListContactsResponse<'a> = ApiResponse<'a, ListContactsResponseData>;


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateContactRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub name: String,
    pub icp_principal: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub notifications_url: Option<String>,
    pub evm_public_address: Option<String>,
    pub seed_phrase: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
    pub is_placeholder: Option<bool>,
}

impl CreateContactRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {

        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_user_id(&self.id.as_ref().unwrap().to_string())?;
        }
        
        // Validate ICP principal
        if self.icp_principal.is_empty() {
            return Err(ValidationError {
                field: "icp_principal".to_string(),
                message: "ICP principal cannot be empty".to_string(),
            });
        }

        // Validate the ICP principal is valid
        match candid::Principal::from_text(&self.icp_principal) {
            Ok(_) => {},
            Err(_) => {
                return Err(ValidationError {
                    field: "icp_principal".to_string(),
                    message: "Invalid ICP principal format".to_string(),
                });
            }
        }

        // Validate name (up to 256 chars)
        validate_id_string(&self.name, "name")?;

        // Validate email if provided
        if let Some(email) = &self.email {
            validate_email(email)?;
        }

        // Validate avatar if provided
        if let Some(avatar) = &self.avatar {
            validate_url(avatar, "avatar")?;
        }

        // Validate notifications_url if provided
        if let Some(notifications_url) = &self.notifications_url {
            validate_url(notifications_url, "notifications_url")?;
        }

        // Validate EVM address if provided
        if let Some(evm_address) = &self.evm_public_address {
            validate_evm_address(evm_address)?;
        }

        if let Some(seed_phrase) = &self.seed_phrase {
            // If a seed phrase is provided, verify that it generates the expected addresses
            match seed_phrase_to_wallet_addresses(seed_phrase) {
                Ok(addresses) => {
                    // Verify that the provided ICP principal matches the one derived from the seed
                    if addresses.icp_principal != self.icp_principal {
                        return Err(ValidationError {
                            field: "seed_phrase".to_string(),
                            message: format!(
                                "Seed phrase generates ICP principal '{}' which doesn't match the provided principal '{}'",
                                addresses.icp_principal, self.icp_principal
                            ),
                        });
                    }
                    
                    // If EVM address is provided, verify it matches the one derived from the seed
                    if let Some(evm_address) = &self.evm_public_address {
                        if &addresses.evm_public_address != evm_address {
                            return Err(ValidationError {
                                field: "seed_phrase".to_string(),
                                message: format!(
                                    "Seed phrase generates EVM address '{}' which doesn't match the provided address '{}'",
                                    addresses.evm_public_address, evm_address
                                ),
                            });
                        }
                    }
                },
                Err(err) => {
                    return Err(ValidationError {
                        field: "seed_phrase".to_string(),
                        message: err.message,
                    });
                }
            }
        }

        // Validate public_note if provided (up to 8192 chars for descriptions)
        if let Some(public_note) = &self.public_note {
            if public_note.len() > 8192 {
                return Err(ValidationError {
                    field: "public_note".to_string(),
                    message: "Public note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate private_note if provided (up to 8192 chars for descriptions)
        if let Some(private_note) = &self.private_note {
            if private_note.len() > 8192 {
                return Err(ValidationError {
                    field: "private_note".to_string(),
                    message: "Private note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }

        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContactRequestBody {
    pub id: UserID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_public_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}

impl UpdateContactRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate the UserID
        validate_user_id(&self.id.0)?;

        // Validate nickname if provided
        if let Some(name) = &self.name {
            validate_id_string(name, "name")?;
        }

        // Validate public_note if provided
        if let Some(public_note) = &self.public_note {
            if public_note.len() > 8192 {
                return Err(ValidationError {
                    field: "public_note".to_string(),
                    message: "Public note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate private_note if provided
        if let Some(private_note) = &self.private_note {
            if private_note.len() > 8192 {
                return Err(ValidationError {
                    field: "private_note".to_string(),
                    message: "Private note must be 8,192 characters or less".to_string(),
                });
            }
        }

        // Validate email if provided
        if let Some(email) = &self.email {
            validate_email(email)?;
        }

        // Validate avatar if provided
        if let Some(avatar) = &self.avatar {
            validate_url(avatar, "avatar")?;
        }

        // Validate notifications_url if provided
        if let Some(notifications_url) = &self.notifications_url {
            validate_url(notifications_url, "notifications_url")?;
        }

        // Validate EVM address if provided
        if let Some(evm_address) = &self.evm_public_address {
            validate_evm_address(evm_address)?;
        }


        // Validate external_id if provided
        if let Some(external_id) = &self.external_id {
            validate_external_id(external_id)?;
        }

        // Validate external_payload if provided
        if let Some(external_payload) = &self.external_payload {
            validate_external_payload(external_payload)?;
        }

        Ok(())
    }
}


pub type CreateContactResponse<'a> = ApiResponse<'a, ContactFE>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContactRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateContactResponse<'a> = ApiResponse<'a, ContactFE>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteContactRequest {
    pub id: UserID,
}
impl DeleteContactRequest {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate the UserID
        validate_user_id(&self.id.0)?;
        
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DeletedContactData {
    pub id: UserID,
    pub deleted: bool
}

pub type DeleteContactResponse<'a> = ApiResponse<'a, DeletedContactData>;


pub type ErrorResponse<'a> = ApiResponse<'a, ()>;


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemContactRequestBody {
    pub current_user_id: String,
    pub new_user_id: String,
    pub redeem_code: String,
}

impl RedeemContactRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        
        // validate user ids
        validate_user_id(&self.new_user_id)?;
        validate_user_id(&self.current_user_id)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedeemContactResponseBody {
    pub contact: ContactFE,
    pub api_key: ApiKeyValue,
}
pub type RedeemContactResponse<'a> = ApiResponse<'a, RedeemContactResponseBody>;
