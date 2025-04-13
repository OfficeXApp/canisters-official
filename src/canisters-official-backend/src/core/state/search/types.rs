// src/core/state/search/types.rs

use core::fmt;

use serde::{Deserialize, Serialize};
use crate::core::{state::{directory::types::{FileID, FolderID}, disks::types::DiskID, drives::types::DriveID, groups::types::GroupID}, types::UserID};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchCategoryEnum {
    All,
    Files,
    Folders,
    Contacts,
    Disks,
    Drives,
    Groups,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResultResourceID {
    File(FileID),
    Folder(FolderID),
    Contact(UserID),
    Disk(DiskID),
    Drive(DriveID),
    Group(GroupID),
}
// implement display
impl fmt::Display for SearchResultResourceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchResultResourceID::File(id) => write!(f, "{}", id),
            SearchResultResourceID::Folder(id) => write!(f, "{}", id),
            SearchResultResourceID::Contact(id) => write!(f, "{}", id),
            SearchResultResourceID::Disk(id) => write!(f, "{}", id),
            SearchResultResourceID::Drive(id) => write!(f, "{}", id),
            SearchResultResourceID::Group(id) => write!(f, "{}", id),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub preview: String,
    pub score: u64,
    pub resource_id: String,
    pub category: SearchCategoryEnum,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: Option<String>,
}

