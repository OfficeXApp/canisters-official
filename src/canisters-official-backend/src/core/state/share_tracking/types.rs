// src/core/state/share_tracking/types.rs

use std::fmt;
use std::str::FromStr;
use std::convert::TryFrom;
use serde::{Serialize, Deserialize};

use crate::core::types::IDPrefix;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShareTrackID(pub String);

impl fmt::Display for ShareTrackID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShareTrackResourceID {
    File(String),
    Folder(String),
}

impl TryFrom<&str> for ShareTrackResourceID {
    type Error = String;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        if id.starts_with(IDPrefix::File.as_str()) {
            Ok(ShareTrackResourceID::File(id.to_string()))
        } else if id.starts_with(IDPrefix::Folder.as_str()) {
            Ok(ShareTrackResourceID::Folder(id.to_string()))
        } else {
            Err(format!("Invalid resource ID prefix, expected {} or {}", 
                IDPrefix::File.as_str(), 
                IDPrefix::Folder.as_str()))
        }
    }
}

impl ShareTrackResourceID {
    pub fn as_str(&self) -> &str {
        match self {
            ShareTrackResourceID::File(id) => id.as_str(),
            ShareTrackResourceID::Folder(id) => id.as_str(),
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, ShareTrackResourceID::File(_))
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, ShareTrackResourceID::Folder(_))
    }
}

impl fmt::Display for ShareTrackResourceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareTrack {
    pub id: ShareTrackID,
    pub origin: Option<ShareTrackID>,
    pub from: Option<UserId>,
    pub to: Option<UserId>,
    pub resource_id: ShareTrackResourceID,
    pub timestamp_ms: u64,
    pub metadata: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadixNode {
    pub node_id: String,
    pub share_track_ids: Vec<ShareTrackID>,
}

impl RadixNode {
    pub fn new(resource_id: &ShareTrackResourceID, referrer_id: Option<&ShareTrackID>) -> Self {
        let node_id = match referrer_id {
            Some(ref_id) => format!("{}_{}", resource_id, ref_id),
            None => resource_id.to_string(),
        };

        RadixNode {
            node_id,
            share_track_ids: Vec::new(),
        }
    }
}