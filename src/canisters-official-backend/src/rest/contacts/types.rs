// src/rest/contacts/types.rs

use serde::{Deserialize, Serialize};

use crate::{core::{state::{contacts::types::Contact, team_invites::types::TeamInviteeID}, types::UserID}, rest::webhooks::types::SortDirection};

#[derive(Debug, Clone, Serialize)]
pub enum ContactResponse<'a, T = ()> {
    #[serde(rename = "ok")]
    Ok { data: &'a T },
    #[serde(rename = "err")]
    Err { code: u16, message: String },
}

impl<'a, T: Serialize> ContactResponse<'a, T> {
    pub fn ok(data: &'a T) -> ContactResponse<'a, T> {
        Self::Ok { data }
    }

    pub fn not_found() -> Self {
        Self::err(404, "Not found".to_string())
    }

    pub fn unauthorized() -> Self {
        Self::err(401, "Unauthorized".to_string())
    }

    pub fn err(code: u16, message: String) -> Self {
        Self::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize value")
    }
}



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


#[derive(Debug, Clone, Serialize)]
pub struct ListContactsResponseData {
    pub items: Vec<Contact>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub type GetContactResponse<'a> = ContactResponse<'a, Contact>;

pub type ListContactsResponse<'a> = ContactResponse<'a, ListContactsResponseData>;


#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertContactRequestBody {
    Create(CreateContactRequestBody),
    Update(UpdateContactRequestBody),
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateContactRequestBody {
    pub icp_principal: String,
    pub nickname: String,
    pub evm_public_address: Option<String>,
    pub public_note: Option<String>,
    pub private_note: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContactRequestBody {
    pub id: String,
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
}


pub type CreateContactResponse<'a> = ContactResponse<'a, Contact>;



#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContactRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

pub type UpdateContactResponse<'a> = ContactResponse<'a, Contact>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteContactRequest {
    pub id: TeamInviteeID,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedContactData {
    pub id: TeamInviteeID,
    pub deleted: bool
}

pub type DeleteContactResponse<'a> = ContactResponse<'a, DeletedContactData>;


pub type ErrorResponse<'a> = ContactResponse<'a, ()>;