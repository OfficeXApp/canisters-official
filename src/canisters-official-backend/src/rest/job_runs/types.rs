use serde::{Deserialize, Serialize};

use crate::{
    core::{
        api::permissions::system::check_system_permissions,
        state::{
            job_runs::types::{JobRun, JobRunID, JobRunStatus}, labels::state::validate_uuid4_string_with_prefix, permissions::types::{PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum}
        },
        types::{ClientSuggestedUUID, IDPrefix, UserID},
    },
    rest::{
        types::{validate_id_string, validate_short_string, validate_unclaimed_uuid, validate_url, ApiResponse, ValidationError},
        webhooks::types::SortDirection,
    },
};

/// Frontend representation of a JobRun, including permission previews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRunFE {
    #[serde(flatten)]
    pub job_run: JobRun,
    pub permission_previews: Vec<SystemPermissionType>,
}

impl JobRunFE {
    /// Redacts sensitive JobRun fields based on user permissions.
    pub fn redacted(&self, user_id: &UserID) -> Self {
        let mut redacted = self.clone();

        // For JobRun, `notes` is strictly internal, `vendor_notes` is for the vendor.
        // Assuming the `vendor_id` is the primary "owner" for specific JobRun data.
        let is_vendor_of_job = user_id == &redacted.job_run.vendor_id;

        // Check if the user has a general view permission (e.g., admin, or role with broad view)
        let has_table_view_permission = redacted.permission_previews.contains(&SystemPermissionType::View);

        // `notes` field is always internal and should be redacted for anyone not considered an "owner" (system owner/admin).
        // For simplicity here, we redact for anyone who isn't the direct vendor of this job run.
        if !is_vendor_of_job && !has_table_view_permission {
            redacted.job_run.notes = "".to_string(); // Always redact 'notes' for non-vendor/non-admin
        }

        // `vendor_notes` and `tracer` are for the vendor, redact for others without view permission
        if !is_vendor_of_job {
            if !has_table_view_permission {
                redacted.job_run.vendor_notes = "".to_string();
                redacted.job_run.tracer = None;
            }
        }

        redacted
    }
}

/// Request body for listing JobRuns.
#[derive(Debug, Clone, Deserialize)]
pub struct ListJobRunsRequestBody {
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

impl ListJobRunsRequestBody {
    /// Validates the fields in the list JobRuns request body.
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        if self.filters.len() > 256 {
            return Err(ValidationError {
                field: "filters".to_string(),
                message: "Filters must be 256 characters or less".to_string(),
            });
        }

        if self.page_size == 0 || self.page_size > 1000 {
            return Err(ValidationError {
                field: "page_size".to_string(),
                message: "Page size must be between 1 and 1000".to_string(),
            });
        }

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

/// Response data for listing JobRuns.
#[derive(Debug, Clone, Serialize)]
pub struct ListJobRunsResponseData {
    pub items: Vec<JobRunFE>,
    pub page_size: usize,
    pub total: usize,
    pub direction: SortDirection,
    pub cursor: Option<String>,
}

/// Request body for creating a new JobRun.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateJobRunRequestBody {
    pub id: Option<ClientSuggestedUUID>,
    pub template_id: Option<String>,
    pub title: String,
    pub vendor_name: String,
    pub vendor_id: UserID,
    pub about_url: String,
    pub status: Option<JobRunStatus>,
    pub description: Option<String>,
    pub billing_url: Option<String>,
    pub support_url: Option<String>,
    pub delivery_url: Option<String>,
    pub verification_url: Option<String>,
    pub installation_url: Option<String>,
    pub subtitle: Option<String>,
    pub pricing: Option<String>,
    pub vendor_notes: Option<String>,
    pub notes: Option<String>,
    pub related_resources: Option<Vec<String>>,
    pub tracer: Option<String>,
    pub labels: Option<Vec<String>>,
    pub external_id: Option<String>,
    pub external_payload: Option<String>,
}

impl CreateJobRunRequestBody {
    /// Validates the fields in the create JobRun request body.
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        if self.id.is_some() {
            validate_unclaimed_uuid(&self.id.as_ref().unwrap().to_string())?;
            validate_uuid4_string_with_prefix(&self.id.as_ref().unwrap().to_string(), IDPrefix::JobRun)?;
        }

        validate_short_string(&self.vendor_name, "vendor_name")?;
        if let Some(description) = &self.description {
            validate_long_string(description, "description", 8192)?;
        }
        validate_short_string(&self.title, "title")?;
        if let Some(notes) = &self.notes {
            validate_long_string(notes, "notes", 8192)?;
        }
        if let Some(billing_url) = &self.billing_url {
            validate_url(billing_url, "billing_url")?;
        }
        if let Some(support_url) = &self.support_url {
            validate_url(support_url, "support_url")?;
        }
        if let Some(delivery_url) = &self.delivery_url {
            validate_url(delivery_url, "delivery_url")?;
        }
        if let Some(verification_url) = &self.verification_url {
            validate_url(verification_url, "verification_url")?;
        }
        if let Some(installation_url) = &self.installation_url {
            validate_url(installation_url, "installation_url")?;
        }

        if let Some(subtitle) = &self.subtitle {
            validate_short_string(subtitle, "subtitle")?;
        }
        if let Some(pricing) = &self.pricing {
            validate_short_string(pricing, "pricing")?;
        }
        if let Some(vendor_notes) = &self.vendor_notes {
            validate_long_string(vendor_notes, "vendor_notes", 8192)?;
        }
        if let Some(tracer) = &self.tracer {
            validate_short_string(tracer, "tracer")?;
        }
        if let Some(related_resources) = &self.related_resources {
            for resource_id in related_resources {
                validate_id_string(resource_id, "related_resource_id")?;
            }
        }

        if let Some(labels) = &self.labels {
            for label in labels {
                validate_id_string(label, "label")?;
            }
        }

        if let Some(external_id) = &self.external_id {
            validate_id_string(external_id, "external_id")?;
        }
        if let Some(external_payload) = &self.external_payload {
            validate_long_string(external_payload, "external_payload", 8192)?;
        }

        Ok(())
    }
}

/// Request body for updating an existing JobRun.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateJobRunRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<JobRunStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_resources: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_payload: Option<String>,
}

impl UpdateJobRunRequestBody {
    /// Validates the fields in the update JobRun request body.
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        validate_id_string(&self.id, "id")?;

        if let Some(url) = &self.billing_url {
            validate_url(url, "billing_url")?;
        }
        if let Some(url) = &self.support_url {
            validate_url(url, "support_url")?;
        }
        if let Some(url) = &self.delivery_url {
            validate_url(url, "delivery_url")?;
        }
        if let Some(url) = &self.verification_url {
            validate_url(url, "verification_url")?;
        }
        if let Some(url) = &self.installation_url {
            validate_url(url, "installation_url")?;
        }

        if let Some(subtitle) = &self.subtitle {
            validate_short_string(subtitle, "subtitle")?;
        }
        if let Some(pricing) = &self.pricing {
            validate_short_string(pricing, "pricing")?;
        }
        if let Some(vendor_notes) = &self.vendor_notes {
            validate_long_string(vendor_notes, "vendor_notes", 8192)?;
        }
        if let Some(tracer) = &self.tracer {
            validate_short_string(tracer, "tracer")?;
        }

        if let Some(resources) = &self.related_resources {
            for resource_id in resources {
                validate_id_string(resource_id, "related_resource_id")?;
            }
        }

        Ok(())
    }
}

// This is a helper validation function that might typically live in `src/rest/types.rs`.
// Included here for completeness as it's used by JobRun types.
fn validate_long_string(value: &str, field_name: &str, max_len: usize) -> Result<(), ValidationError> {
    if value.len() > max_len {
        return Err(ValidationError {
            field: field_name.to_string(),
            message: format!("{} must be {} characters or less", field_name, max_len),
        });
    }
    Ok(())
}

/// Request body for deleting a JobRun.
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteJobRunRequest {
    pub id: JobRunID,
}
impl DeleteJobRunRequest {
    /// Validates the fields in the delete JobRun request body.
    pub fn validate_body(&self) -> Result<(), ValidationError> {
        validate_id_string(&self.id.0, "id")?;
        let prefix = IDPrefix::JobRun.as_str();
        if !self.id.0.starts_with(prefix) {
            return Err(ValidationError {
                field: "id".to_string(),
                message: format!("JobRun ID must start with '{}'", prefix),
            });
        }
        Ok(())
    }
}

/// Response data after deleting a JobRun.
#[derive(Debug, Clone, Serialize)]
pub struct DeletedJobRunData {
    pub id: JobRunID,
    pub deleted: bool,
}

// Type aliases for API responses
pub type GetJobRunResponse<'a> = ApiResponse<'a, JobRunFE>;
pub type DeleteJobRunResponse<'a> = ApiResponse<'a, DeletedJobRunData>;
pub type ErrorResponse<'a> = ApiResponse<'a, ()>;
pub type ListJobRunsResponse<'a> = ApiResponse<'a, ListJobRunsResponseData>;
pub type CreateJobRunResponse<'a> = ApiResponse<'a, JobRunFE>;
pub type UpdateJobRunResponse<'a> = ApiResponse<'a, JobRunFE>;