// src/rest/mod.rs

// general imports
pub mod router; 
pub mod helpers;
pub mod auth;
pub mod types;

// rest route imports
pub mod templates;  
pub mod api_keys;
pub mod webhooks;
pub mod contacts;
pub mod groups;
pub mod group_invites;
pub mod drives;
pub mod disks;
pub mod directory;
pub mod permissions;
pub mod tags;
pub mod organization;