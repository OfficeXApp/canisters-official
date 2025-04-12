// src/rest/giftcards/types.rs

use candid::CandidType;
use serde::{Deserialize, Serialize};
use crate::{
    core::{
        state::giftcards_refuel::types::{FactoryRefuelHistoryRecord, GiftcardRefuel, GiftcardRefuelID}, 
        types::{ICPPrincipalString, IDPrefix, UserID}
    }, 
    rest::types::{
            validate_external_id, validate_external_payload, validate_icp_principal, validate_id_string, validate_user_id, ApiResponse, UpsertActionTypeEnum, ValidationError
        }
};



#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}



// Add pagination request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGiftcardRefuelsRequestBody {
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

impl ListGiftcardRefuelsRequestBody {
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
pub struct ListGiftcardRefuelsResponseData {
    pub items: Vec<GiftcardRefuel>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGiftcardRefuelRequestBody {
    pub action: UpsertActionTypeEnum,
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub external_id: String,
}
impl CreateGiftcardRefuelRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        
        // validate gas_cycles_included (must be greater than 1T)
        if self.gas_cycles_included < 1_000_000_000_000 {
            return Err(ValidationError {
                field: "gas_cycles_included".to_string(),
                message: "Gas cycles included must be greater than 1T".to_string(),
            });
        }

        // action must be UpsertActionTypeEnum::Create
        if self.action != UpsertActionTypeEnum::Create {
            return Err(ValidationError {
                field: "action".to_string(),
                message: "Action must be 'Create'".to_string(),
            });
        }

        Ok(())
    }
}
pub type CreateGiftcardRefuelResponse<'a> = ApiResponse<'a, GiftcardRefuel>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteGiftcardRefuelRequestBody {
    pub id: String,
}
impl DeleteGiftcardRefuelRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::GiftcardRefuel.as_str();
        if !self.id.starts_with(api_key_prefix) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("API Key ID must start with '{}'", api_key_prefix),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedGiftcardRefuelData {
    pub id: String,
    pub deleted: bool
}
pub type DeleteGiftcardRefuelResponse<'a> = ApiResponse<'a, DeletedGiftcardRefuelData>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGiftcardRefuelRequestBody {
    pub action: UpsertActionTypeEnum,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd_revenue_cents: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_cycles_included: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}
impl UpdateGiftcardRefuelRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars, and start with GiftcardRefuelID_ prefix)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::GiftcardRefuel.as_str();
        if !self.id.starts_with(api_key_prefix) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("API Key ID must start with '{}'", api_key_prefix),
            });
        }

        // action must be UpsertActionTypeEnum::Update
        if self.action != UpsertActionTypeEnum::Update {
            return Err(ValidationError {
                field: "action".to_string(),
                message: "Action must be 'Update'".to_string(),
            });
        }

        // validate gas_cycles_included (must be greater than 1T)
        if let Some(gas_cycles_included) = self.gas_cycles_included {
            if gas_cycles_included < 1_000_000_000_000 {
                return Err(ValidationError {
                    field: "gas_cycles_included".to_string(),
                    message: "Gas cycles included must be greater than 1T".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertGiftcardRefuelRequestBody {
    Create(CreateGiftcardRefuelRequestBody),
    Update(UpdateGiftcardRefuelRequestBody),
}
impl UpsertGiftcardRefuelRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertGiftcardRefuelRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertGiftcardRefuelRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

pub type UpdateGiftcardRefuelResponse<'a> = ApiResponse<'a, GiftcardRefuel>;
pub type ListGiftcardRefuelsResponse<'a> = ApiResponse<'a, ListGiftcardRefuelsResponseData>;
pub type GetGiftcardRefuelResponse<'a> = ApiResponse<'a, GiftcardRefuel>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemGiftcardRefuelData {
    pub giftcard_id: GiftcardRefuelID,
    pub icp_principal: String,
}
impl RedeemGiftcardRefuelData {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate giftcard_id format
        if !self.giftcard_id.0.starts_with(IDPrefix::GiftcardRefuel.as_str()) {
            return Err(ValidationError {
                field: "giftcard_id".to_string(),
                message: format!("GiftcardRefuel ID must start with '{}'", IDPrefix::GiftcardRefuel.as_str()),
            });
        }

        // Validate ICP principal
        match validate_icp_principal(&self.icp_principal) {
            Ok(_) => {},
            Err(validation_error) => {
                return Err(ValidationError {
                    field: "icp_principal".to_string(),
                    message: validation_error.message,
                });
            }
        };

        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemGiftcardRefuelResult {
    pub giftcard_id: GiftcardRefuelID,
    pub icp_principal: String,
    pub redeem_code: String,
    pub timestamp_ms: u64
}

pub type RedeemGiftcardRefuelResponse<'a> = ApiResponse<'a, RedeemGiftcardRefuelResult>;
