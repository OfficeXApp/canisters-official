// src/rest/vouchers/types.rs

use candid::CandidType;
use serde::{Deserialize, Serialize};
use crate::{
    core::{
        state::vouchers::types::{FactorySpawnHistoryRecord, Voucher, VoucherID}, 
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
pub struct ListVouchersRequestBody {
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

impl ListVouchersRequestBody {
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
pub struct ListVouchersResponseData {
    pub items: Vec<Voucher>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateVoucherRequestBody {
    pub action: UpsertActionTypeEnum,
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub external_id: String,
}
impl CreateVoucherRequestBody {
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
pub type CreateVoucherResponse<'a> = ApiResponse<'a, Voucher>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteVoucherRequestBody {
    pub id: String,
}
impl DeleteVoucherRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::Voucher.as_str();
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
pub struct DeletedVoucherData {
    pub id: String,
    pub deleted: bool
}
pub type DeleteVoucherResponse<'a> = ApiResponse<'a, DeletedVoucherData>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVoucherRequestBody {
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
impl UpdateVoucherRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars, and start with VoucherID_ prefix)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::Voucher.as_str();
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
pub enum UpsertVoucherRequestBody {
    Create(CreateVoucherRequestBody),
    Update(UpdateVoucherRequestBody),
}
impl UpsertVoucherRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertVoucherRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertVoucherRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

pub type UpdateVoucherResponse<'a> = ApiResponse<'a, Voucher>;
pub type ListVouchersResponse<'a> = ApiResponse<'a, ListVouchersResponseData>;
pub type GetVoucherResponse<'a> = ApiResponse<'a, Voucher>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemVoucherData {
    pub id: VoucherID,
    pub owner_icp_principal: String,
    pub nickname: Option<String>
}
impl RedeemVoucherData {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate voucher_id format
        if !self.id.0.starts_with(IDPrefix::Voucher.as_str()) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("Voucher ID must start with '{}'", IDPrefix::Voucher.as_str()),
            });
        }

        // Validate owner ICP principal
        match validate_icp_principal(&self.owner_icp_principal) {
            Ok(_) => {},
            Err(validation_error) => {
                return Err(ValidationError {
                    field: "owner_icp_principal".to_string(),
                    message: validation_error.message,
                });
            }
        };

        // Validate nickname if provided
        if let Some(nickname) = &self.nickname {
            if nickname.trim().is_empty() {
                return Err(ValidationError {
                    field: "nickname".to_string(),
                    message: "Nickname cannot be empty".to_string(),
                });
            }

            if nickname.len() > 64 {
                return Err(ValidationError {
                    field: "nickname".to_string(),
                    message: "Nickname must be 64 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}

pub type RedeemVoucherResponse<'a> = ApiResponse<'a, FactorySpawnHistoryRecord>;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct SpawnInitArgs {
    pub owner: String, // Plain string for simplicity, really should be ICPPrincipalString
    pub nickname: Option<String>,
    pub note: Option<String>,
    pub spawn_redeem_code: Option<String>,
}