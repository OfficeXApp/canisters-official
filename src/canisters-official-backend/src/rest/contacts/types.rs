// src/rest/contacts/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::{state::{contacts::types::Contact, team_invites::types::TeamInviteeID}, types::UserID}, rest::{types::{validate_evm_address, validate_external_id, validate_external_payload, validate_id_string, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError}, webhooks::types::SortDirection}};


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
    pub icp_principal: String,
    pub nickname: String,
    pub evm_public_address: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
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

        // Validate nickname (up to 256 chars)
        validate_id_string(&self.nickname, "nickname")?;

        // Validate EVM address if provided
        if let Some(evm_address) = &self.evm_public_address {
            validate_evm_address(evm_address)?;
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
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_public_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icp_principal: Option<String>,
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
        if let Some(nickname) = &self.nickname {
            validate_id_string(nickname, "nickname")?;
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