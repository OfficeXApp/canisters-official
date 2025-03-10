// src/core/state/search/types.rs

use serde::{Deserialize, Serialize};
use crate::core::{state::{directory::types::{FileID, FolderID}, disks::types::DiskID, drives::types::DriveID, teams::types::TeamID}, types::UserID};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchCategoryEnum {
    All,
    Files,
    Folders,
    Contacts,
    Disks,
    Drives,
    Teams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResultResourceID {
    File(FileID),
    Folder(FolderID),
    Contact(UserID),
    Disk(DiskID),
    Drive(DriveID),
    Team(TeamID),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub preview: String,
    pub score: u64,
    pub resource_id: SearchResultResourceID,
    pub category: SearchCategoryEnum,
}

