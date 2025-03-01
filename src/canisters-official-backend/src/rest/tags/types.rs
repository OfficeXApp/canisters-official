// src/rest/tags/types.rs

use serde::{Deserialize, Serialize};
use crate::core::state::tags::types::{Tag, TagID, TagResourceID};
use crate::rest::webhooks::types::SortDirection;


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTagsRequestBody {
    #[serde(default)]
    pub filters: ListTagsRequestBodyFilters,
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



#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTagsRequestBodyFilters {
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListTagsResponseData {
    pub items: Vec<Tag>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UpsertTagRequestBody {
    Create(CreateTagRequestBody),
    Update(UpdateTagRequestBody),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTagRequestBody {
    pub value: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTagRequestBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTagRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeletedTagData {
    pub id: TagID,
    pub deleted: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagResourceRequest {
    pub tag_id: String,
    pub resource_id: String,
    pub add: bool,  // true to add, false to remove
}

#[derive(Debug, Clone, Serialize)]
pub struct TagOperationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub tag: Option<Tag>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTagResourcesRequest {
    pub tag_id: String,
    pub resource_type: Option<String>,
    pub page_size: Option<usize>,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTagResourcesResponseData {
    pub tag_id: String,
    pub resources: Vec<TagResourceID>,
    pub page_size: usize,
    pub total: usize,
    pub cursor_up: Option<String>,
    pub cursor_down: Option<String>,
}

pub type GetTagResponse<'a> = TagResponse<'a, Tag>;
pub type DeleteTagResponse<'a> = TagResponse<'a, DeletedTagData>;
pub type ErrorResponse<'a> = TagResponse<'a, ()>;
pub type ListTagsResponse<'a> = TagResponse<'a, ListTagsResponseData>;
pub type CreateTagResponse<'a> = TagResponse<'a, Tag>;
pub type UpdateTagResponse<'a> = TagResponse<'a, Tag>;
pub type TagResourceResponse<'a> = TagResponse<'a, TagOperationResponse>;
pub type GetTagResourcesResponse<'a> = TagResponse<'a, GetTagResourcesResponseData>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TagResponse<'a, T>
where
    T: Serialize,
{
    Ok { data: &'a T },
    Err { code: u16, message: String },
}

impl<'a, T> TagResponse<'a, T>
where
    T: Serialize,
{
    pub fn ok(data: &'a T) -> Self {
        TagResponse::Ok { data }
    }

    pub fn err(code: u16, message: String) -> Self {
        TagResponse::Err { code, message }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_else(|_| 
            serde_json::to_vec(&TagResponse::Err::<()> {
                code: 500,
                message: "Failed to serialize response".to_string(),
            }).unwrap_or_default()
        )
    }
}

impl<'a> TagResponse<'a, ()> {
    pub fn not_found() -> Self {
        TagResponse::Err {
            code: 404,
            message: "Not found".to_string(),
        }
    }
}