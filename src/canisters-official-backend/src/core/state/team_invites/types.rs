// src/core/state/team_invites/types.rs
use serde::{Serialize, Deserialize};

use crate::core::state::teams::types::TeamID;
use crate::core::types::{UserID};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamInviteID(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Team_Invite {
    pub id: TeamInviteID,
    pub team_id: TeamID,
    pub inviter_id: UserID,
    pub invitee_id: UserID,
    pub role: TeamRole,
    pub active_from: u64,
    pub expires_at: i64,
    pub created_at: u64,
    pub last_modified_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TeamRole {
    Admin,
    Member
}
