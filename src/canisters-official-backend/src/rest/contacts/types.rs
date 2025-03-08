// src/rest/contacts/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::{state::{contacts::types::Contact, team_invites::types::TeamInviteeID}, types::{IDPrefix, UserID}}, rest::{auth::seed_phrase_to_wallet_addresses, types::{validate_email, validate_evm_address, validate_external_id, validate_external_payload, validate_id_string, validate_url, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};


#[derive(Debug, Clone, Deserialize)]
pub struct ListContactsRequestBody {
    #[serde(default)]
    pub filters: String,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub direction: SortDirection,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
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
        if let Some(cursor) = &self.cursor_up {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_up".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        if let Some(cursor) = &self.cursor_down {
            if cursor.len() > 256 {
                return Err(ValidationError {
                    field: "cursor_down".to_string(),
                    message: "Cursor must be 256 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct ListContactsResponseData {
    pub items: Vec<Contact>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub type GetContactResponse<'a> = ApiResponse<'a, Contact>;

pub type ListContactsResponse<'a> = ApiResponse<'a, ListContactsResponseData>;


#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertContactRequestBody {
    Create(CreateContactRequestBody),
    Update(UpdateContactRequestBody),
}

impl UpsertContactRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertContactRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertContactRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateContactRequestBody {
    pub action: UpsertActionTypeEnum,
    pub name: String,
    pub icp_principal: String,
    pub email: Option<String>,
    pub webhook_url: Option<String>,
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

        // Validate webhook_url if provided
        if let Some(webhook_url) = &self.webhook_url {
            validate_url(webhook_url, "webhook_url")?;
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
    pub action: UpsertActionTypeEnum,
    pub id: UserID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_public_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icp_principal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_phrase: Option<String>,
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

        // Validate webhook_url if provided
        if let Some(webhook_url) = &self.webhook_url {
            validate_url(webhook_url, "webhook_url")?;
        }

        // Validate EVM address if provided
        if let Some(evm_address) = &self.evm_public_address {
            validate_evm_address(evm_address)?;
        }

        // Validate ICP principal if provided
        if let Some(icp_principal) = &self.icp_principal {
            if icp_principal.is_empty() {
                return Err(ValidationError {
                    field: "icp_principal".to_string(),
                    message: "ICP principal cannot be empty".to_string(),
                });
            }

            match candid::Principal::from_text(icp_principal) {
                Ok(_) => {},
                Err(_) => {
                    return Err(ValidationError {
                        field: "icp_principal".to_string(),
                        message: "Invalid ICP principal format".to_string(),
                    });
                }
            }
        }

        if let Some(seed_phrase) = &self.seed_phrase {
            // If a seed phrase is provided, both ICP principal and EVM address must also be provided
            if self.icp_principal.is_none() {
                return Err(ValidationError {
                    field: "icp_principal".to_string(),
                    message: "When seed phrase is provided, ICP principal must also be provided".to_string(),
                });
            }

            if self.evm_public_address.is_none() {
                return Err(ValidationError {
                    field: "evm_public_address".to_string(),
                    message: "When seed phrase is provided, EVM address must also be provided".to_string(),
                });
            }

            // Now validate the seed phrase generates the expected addresses
            match seed_phrase_to_wallet_addresses(seed_phrase) {
                Ok(addresses) => {
                    // Verify ICP principal matches
                    if let Some(icp_principal) = &self.icp_principal {
                        if &addresses.icp_principal != icp_principal {
                            return Err(ValidationError {
                                field: "seed_phrase".to_string(),
                                message: format!(
                                    "Seed phrase generates ICP principal '{}' which doesn't match the provided principal '{}'",
                                    addresses.icp_principal, icp_principal
                                ),
                            });
                        }
                    }

                    // Verify EVM address matches
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

                    // Extract principal from UserID (removing the prefix)
                    let user_prefix = IDPrefix::User.as_str();
                    if self.id.0.starts_with(user_prefix) {
                        let user_principal = &self.id.0[user_prefix.len()..];
                        
                        // Extract principal from the derived ICP principal
                        // The derived ICP principal doesn't have the prefix, so we compare directly
                        if addresses.icp_principal != user_principal {
                            return Err(ValidationError {
                                field: "seed_phrase".to_string(),
                                message: format!(
                                    "Seed phrase generates ICP principal '{}' which doesn't match the user ID principal '{}'",
                                    addresses.icp_principal, user_principal
                                ),
                            });
                        }
                    } else {
                        return Err(ValidationError {
                            field: "id".to_string(),
                            message: format!(
                                "User ID '{}' doesn't start with the expected prefix '{}'",
                                self.id.0, user_prefix
                            ),
                        });
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


pub type CreateContactResponse<'a> = ApiResponse<'a, Contact>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContactRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateContactResponse<'a> = ApiResponse<'a, Contact>;

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
    pub redeem_token: String,
}

impl RedeemContactRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        
        // validate user ids
        validate_user_id(&self.new_user_id)?;
        validate_user_id(&self.current_user_id)?;

        Ok(())
    }
}
