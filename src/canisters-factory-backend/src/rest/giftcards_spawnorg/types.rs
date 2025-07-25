// src/rest/giftcards/types.rs

use candid::CandidType;
use serde::{Deserialize, Serialize};
use crate::{
    core::{
        state::giftcards_spawnorg::types::{DriveID, DriveRESTUrlEndpoint, FactorySpawnHistoryRecord, GiftcardSpawnOrg, GiftcardSpawnOrgID}, 
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
pub struct ListGiftcardSpawnOrgsRequestBody {
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

impl ListGiftcardSpawnOrgsRequestBody {
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
pub struct ListGiftcardSpawnOrgsResponseData {
    pub items: Vec<GiftcardSpawnOrg>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGiftcardSpawnOrgRequestBody {
    pub action: UpsertActionTypeEnum, 
    pub usd_revenue_cents: u64,
    pub note: String,
    pub gas_cycles_included: u64,
    pub external_id: Option<String>,
    pub disk_auth_json: Option<String>,
}
impl CreateGiftcardSpawnOrgRequestBody {
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
pub type CreateGiftcardSpawnOrgResponse<'a> = ApiResponse<'a, GiftcardSpawnOrg>;

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteGiftcardSpawnOrgRequestBody {
    pub id: String,
}
impl DeleteGiftcardSpawnOrgRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::GiftcardSpawnOrg.as_str();
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
pub struct DeletedGiftcardSpawnOrgData {
    pub id: String,
    pub deleted: bool
}
pub type DeleteGiftcardSpawnOrgResponse<'a> = ApiResponse<'a, DeletedGiftcardSpawnOrgData>;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGiftcardSpawnOrgRequestBody {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_auth_json: Option<String>,
}
impl UpdateGiftcardSpawnOrgRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate id (must not be empty, up to 256 chars, and start with GiftcardSpawnOrgID_ prefix)
        validate_id_string(&self.id, "id")?;
        
        // Check if ID has the correct prefix
        let api_key_prefix = IDPrefix::GiftcardSpawnOrg.as_str();
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
pub enum UpsertGiftcardSpawnOrgRequestBody {
    Create(CreateGiftcardSpawnOrgRequestBody),
    Update(UpdateGiftcardSpawnOrgRequestBody),
}
impl UpsertGiftcardSpawnOrgRequestBody {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        match self {
            UpsertGiftcardSpawnOrgRequestBody::Create(create_req) => create_req.validate_body(),
            UpsertGiftcardSpawnOrgRequestBody::Update(update_req) => update_req.validate_body(),
        }
    }
}

pub type UpdateGiftcardSpawnOrgResponse<'a> = ApiResponse<'a, GiftcardSpawnOrg>;
pub type ListGiftcardSpawnOrgsResponse<'a> = ApiResponse<'a, ListGiftcardSpawnOrgsResponseData>;
pub type GetGiftcardSpawnOrgResponse<'a> = ApiResponse<'a, GiftcardSpawnOrg>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemGiftcardSpawnOrgData {
    pub giftcard_id: GiftcardSpawnOrgID,
    pub owner_icp_principal: String,
    pub owner_name: Option<String>,
    pub organization_name: Option<String>
}
impl RedeemGiftcardSpawnOrgData {
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        // Validate giftcard_id format
        if !self.giftcard_id.0.starts_with(IDPrefix::GiftcardSpawnOrg.as_str()) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("GiftcardSpawnOrg ID must start with '{}'", IDPrefix::GiftcardSpawnOrg.as_str()),
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

        // Validate title if provided
        if let Some(organization_name) = &self.organization_name {
            if organization_name.trim().is_empty() {
                return Err(ValidationError {
                    field: "organization_name".to_string(),
                    message: "Org Name cannot be empty".to_string(),
                });
            }

            if organization_name.len() > 64 {
                return Err(ValidationError {
                    field: "organization_name".to_string(),
                    message: "Org Name must be 64 characters or less".to_string(),
                });
            }
        }
        if let Some(owner_name) = &self.owner_name {
            if owner_name.trim().is_empty() {
                return Err(ValidationError {
                    field: "owner_name".to_string(),
                    message: "Org Name cannot be empty".to_string(),
                });
            }

            if owner_name.len() > 64 {
                return Err(ValidationError {
                    field: "owner_name".to_string(),
                    message: "Org Name must be 64 characters or less".to_string(),
                });
            }
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemGiftcardSpawnOrgResult {
    pub owner_id: UserID,
    pub drive_id: DriveID,
    pub endpoint: DriveRESTUrlEndpoint,
    pub redeem_code: String,
    pub disk_auth_json: Option<String>,
}

pub type RedeemGiftcardSpawnOrgResponse<'a> = ApiResponse<'a, RedeemGiftcardSpawnOrgResult>;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct SpawnInitArgs {
    pub owner: String, // Plain string for simplicity, really should be ICPPrincipalString
    pub title: Option<String>,
    pub owner_name: Option<String>,
    pub note: Option<String>,
    pub spawn_redeem_code: Option<String>,
}