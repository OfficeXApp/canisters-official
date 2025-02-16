// src/core/state/teams/types.rs
use serde::{Serialize, Deserialize};
use std::fmt;
use crate::core::{
    state::{drives::types::{DriveID, DriveRESTUrlEndpoint}, team_invites::types::TeamInviteID},
    types::UserID
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamID(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Team {
    pub id: TeamID,
    pub name: String,
    pub owner: UserID,
    pub private_note: Option<String>,
    pub public_note: Option<String>,
    pub admin_invites: Vec<TeamInviteID>, // all admin_invites are also in member_invites
    pub member_invites: Vec<TeamInviteID>,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub drive_id: DriveID,
    pub url_endpoint: DriveRESTUrlEndpoint,
}

// Implement Display for TeamID
impl fmt::Display for TeamID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement Display for Team
impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Team {{ id: {}, name: {}, owner: {} }}", 
            self.id, self.name, self.owner)
    }
}