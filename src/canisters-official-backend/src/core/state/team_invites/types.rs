use std::fmt;

// src/core/state/team_invites/types.rs
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use crate::core::state::teams::types::TeamID;
use crate::core::types::{UserID};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct TeamInviteID(pub String);
impl fmt::Display for TeamInviteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, SerdeDiff)]
pub struct Team_Invite {
    pub id: TeamInviteID,
    pub team_id: TeamID,
    pub inviter_id: UserID,
    pub invitee_id: TeamInviteeID,
    pub role: TeamRole,
    pub note: String,
    pub active_from: u64,
    pub expires_at: i64,
    pub created_at: u64,
    pub last_modified_at: u64,
    pub from_placeholder_invitee: Option<PlaceholderTeamInviteeID>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SerdeDiff)]
pub enum TeamRole {
    Admin,
    Member
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub struct PlaceholderTeamInviteeID(pub String);
impl fmt::Display for PlaceholderTeamInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SerdeDiff)]
pub enum TeamInviteeID {
    User(UserID),
    PlaceholderTeamInvitee(PlaceholderTeamInviteeID),
}
impl fmt::Display for TeamInviteeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TeamInviteeID::User(user_id) => write!(f, "{}", user_id),
            TeamInviteeID::PlaceholderTeamInvitee(placeholder_id) => write!(f, "{}", placeholder_id),
        }
    }
}