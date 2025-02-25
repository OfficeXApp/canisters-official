// src/core/api/replay/diff.rs

use serde_diff::Apply;
use serde_diff::{Diff, SerdeDiff};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use crate::{core::{api::{uuid::update_checksum_for_state_diff, webhooks::state_diffs::{fire_state_diff_webhooks, get_active_state_diff_webhooks}}, state::{api_keys::{state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue}}, contacts::{state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, types::Contact}, directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FileMetadata, FileUUID, FolderMetadata, FolderUUID}}, disks::{state::state::{DISKS_BY_EXTERNAL_ID_HASHTABLE, DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, types::{Disk, DiskID}}, drives::{state::state::{CANISTER_ID, DRIVES_BY_ID_HASHTABLE, DRIVES_BY_TIME_LIST, DRIVE_ID, DRIVE_STATE_TIMESTAMP_NS, OWNER_ID, URL_ENDPOINT}, types::{Drive, DriveID, DriveRESTUrlEndpoint, DriveStateDiffString}}, permissions::{state::state::{DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE, DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE, DIRECTORY_PERMISSIONS_BY_TIME_LIST, SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE, SYSTEM_PERMISSIONS_BY_TIME_LIST}, types::{DirectoryPermission, DirectoryPermissionID, PermissionGranteeID, SystemPermission, SystemPermissionID, SystemResourceID}}, team_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::{TeamInviteID, TeamInviteeID, Team_Invite}}, teams::{state::state::{TEAMS_BY_ID_HASHTABLE, TEAMS_BY_TIME_LIST}, types::{Team, TeamID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE, WEBHOOKS_BY_TIME_LIST}, types::{Webhook, WebhookAltIndexID, WebhookID}}}, types::{PublicKeyICP, UserID}}, rest::directory::types::DirectoryResourceID};

// Define a type to represent the entire state
#[derive(SerdeDiff, Serialize, Deserialize, Clone,)]
pub struct EntireState {
    // About
    DRIVE_ID: DriveID,
    CANISTER_ID: PublicKeyICP,
    OWNER_ID: UserID,
    URL_ENDPOINT: DriveRESTUrlEndpoint,
    DRIVE_STATE_TIMESTAMP_NS: u64,
    // Api Keys
    APIKEYS_BY_VALUE_HASHTABLE: HashMap<ApiKeyValue, ApiKeyID>,
    APIKEYS_BY_ID_HASHTABLE: HashMap<ApiKeyID, ApiKey>,
    USERS_APIKEYS_HASHTABLE: HashMap<UserID, Vec<ApiKeyID>>,
    // Contacts
    CONTACTS_BY_ID_HASHTABLE: HashMap<UserID, Contact>,
    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: HashMap<String, UserID>,
    CONTACTS_BY_TIME_LIST: Vec<UserID>,
    // Directory
    folder_uuid_to_metadata: HashMap<FolderUUID, FolderMetadata>,
    file_uuid_to_metadata: HashMap<FileUUID, FileMetadata>,
    full_folder_path_to_uuid: HashMap<DriveFullFilePath, FolderUUID>,
    full_file_path_to_uuid: HashMap<DriveFullFilePath, FileUUID>,
    // Disks
    DISKS_BY_ID_HASHTABLE: HashMap<DiskID, Disk>,
    DISKS_BY_EXTERNAL_ID_HASHTABLE: HashMap<String, DiskID>,
    DISKS_BY_TIME_LIST: Vec<DiskID>,
    // Drives
    DRIVES_BY_ID_HASHTABLE: HashMap<DriveID, Drive>,
    DRIVES_BY_TIME_LIST: Vec<DriveID>,
    // Permissions
    DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE: HashMap<DirectoryPermissionID, DirectoryPermission>,
    DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE: HashMap<DirectoryResourceID, Vec<DirectoryPermissionID>>,
    DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE: HashMap<PermissionGranteeID, Vec<DirectoryPermissionID>>,
    DIRECTORY_PERMISSIONS_BY_TIME_LIST: Vec<DirectoryPermissionID>,
    SYSTEM_PERMISSIONS_BY_ID_HASHTABLE: HashMap<SystemPermissionID, SystemPermission>,
    SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE: HashMap<SystemResourceID, Vec<SystemPermissionID>>,
    SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE: HashMap<PermissionGranteeID, Vec<SystemPermissionID>>,
    SYSTEM_PERMISSIONS_BY_TIME_LIST: Vec<SystemPermissionID>,
    // Team Invites
    INVITES_BY_ID_HASHTABLE: HashMap<TeamInviteID, Team_Invite>,
    USERS_INVITES_LIST_HASHTABLE: HashMap<TeamInviteeID, Vec<TeamInviteID>>,
    // Teams 
    TEAMS_BY_ID_HASHTABLE: HashMap<TeamID, Team>,
    TEAMS_BY_TIME_LIST: Vec<TeamID>,
    // Webhooks
    WEBHOOKS_BY_ALT_INDEX_HASHTABLE: HashMap<WebhookAltIndexID, Vec<WebhookID>>,
    WEBHOOKS_BY_ID_HASHTABLE: HashMap<WebhookID, Webhook>,
    WEBHOOKS_BY_TIME_LIST: Vec<WebhookID>,
}

fn snapshot_entire_state() -> EntireState {
    EntireState {
        // About
        DRIVE_ID: DRIVE_ID.with(|drive_id| drive_id.clone()),
        CANISTER_ID: CANISTER_ID.with(|canister_id| canister_id.clone()),
        OWNER_ID: OWNER_ID.with(|owner_id| owner_id.borrow().clone()),
        URL_ENDPOINT: URL_ENDPOINT.with(|url| url.borrow().clone()),
        DRIVE_STATE_TIMESTAMP_NS: DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get()),
        // Api Keys
        APIKEYS_BY_VALUE_HASHTABLE: APIKEYS_BY_VALUE_HASHTABLE.with(|store| store.borrow().clone()),
        APIKEYS_BY_ID_HASHTABLE: APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        USERS_APIKEYS_HASHTABLE: USERS_APIKEYS_HASHTABLE.with(|store| store.borrow().clone()),
        // Contacts
        CONTACTS_BY_ID_HASHTABLE: CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| store.borrow().clone()),
        CONTACTS_BY_TIME_LIST: CONTACTS_BY_TIME_LIST.with(|store| store.borrow().clone()),
        // Directory
        folder_uuid_to_metadata: folder_uuid_to_metadata.with(|store| store.clone()),
        file_uuid_to_metadata: file_uuid_to_metadata.with(|store| store.clone()),
        full_folder_path_to_uuid: full_folder_path_to_uuid.with(|store| store.clone()),
        full_file_path_to_uuid: full_file_path_to_uuid.with(|store| store.clone()),
        // Disks
        DISKS_BY_ID_HASHTABLE: DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        DISKS_BY_EXTERNAL_ID_HASHTABLE: DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|store| store.borrow().clone()),
        DISKS_BY_TIME_LIST: DISKS_BY_TIME_LIST.with(|store| store.borrow().clone()),
        // Drives
        DRIVES_BY_ID_HASHTABLE: DRIVES_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        DRIVES_BY_TIME_LIST: DRIVES_BY_TIME_LIST.with(|store| store.borrow().clone()),
        // Permissions
        DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE: DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE: DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|store| store.borrow().clone()),
        DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE: DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|store| store.borrow().clone()),
        DIRECTORY_PERMISSIONS_BY_TIME_LIST: DIRECTORY_PERMISSIONS_BY_TIME_LIST.with(|store| store.borrow().clone()),
        SYSTEM_PERMISSIONS_BY_ID_HASHTABLE: SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE: SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|store| store.borrow().clone()),
        SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE: SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|store| store.borrow().clone()),
        SYSTEM_PERMISSIONS_BY_TIME_LIST: SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|store| store.borrow().clone()),
        // Team Invites
        INVITES_BY_ID_HASHTABLE: INVITES_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        USERS_INVITES_LIST_HASHTABLE: USERS_INVITES_LIST_HASHTABLE.with(|store| store.borrow().clone()),
        // Teams
        TEAMS_BY_ID_HASHTABLE: TEAMS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        TEAMS_BY_TIME_LIST: TEAMS_BY_TIME_LIST.with(|store| store.borrow().clone()),
        // Webhooks
        WEBHOOKS_BY_ALT_INDEX_HASHTABLE: WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| store.borrow().clone()),
        WEBHOOKS_BY_ID_HASHTABLE: WEBHOOKS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        WEBHOOKS_BY_TIME_LIST: WEBHOOKS_BY_TIME_LIST.with(|store| store.borrow().clone()),
    }
}

pub fn snapshot_prestate() -> Option<EntireState> {
    let diff_webhooks = get_active_state_diff_webhooks();
    if diff_webhooks.is_empty() {
        return None
    }
    let before_state = snapshot_entire_state();
    Some(before_state)
}

pub fn snapshot_poststate(before_snapshot: Option<EntireState>, notes: Option<String>) {
    match before_snapshot {
        Some(before_snapshot) => {
            let after_snapshot = snapshot_entire_state();
            let diff = diff_entire_state(before_snapshot, after_snapshot);
            match diff {
                Some(diff) => {
                    fire_state_diff_webhooks(diff, notes);
                },
                None => ()
            }
        },
        None => ()
    }
}

pub fn diff_entire_state(before_snapshot: EntireState, after_snapshot: EntireState) -> Option<DriveStateDiffString> {
    // Create MessagePack diff (minimal size)
    let diff_data = match rmp_serde::to_vec_named(&Diff::serializable(&before_snapshot, &after_snapshot)) {
        Ok(data) => data,
        Err(e) => {
            ic_cdk::println!("Failed to serialize state diff: {}", e);
            Vec::new()
        }
    };

    if diff_data.len() <= 4 {  // Adjust this threshold based on testing
        return None;  // No meaningful difference, skip firing
    }
    
    // Convert diff to base64 for transmission if needed
    let diff_base64 = base64::encode(&diff_data);
    Some(DriveStateDiffString(diff_base64))
}

pub fn apply_state_diff(diff_data: &DriveStateDiffString) -> Result<(), String> {
    // Decode the base64 encoded diff
    let diff_bytes = match BASE64.decode(&diff_data.0) {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("Failed to decode base64 diff: {}", e)),
    };

    // Take a snapshot of the current state
    let mut current_state = snapshot_entire_state();

    // Create a deserializer for the MessagePack data
    let mut deserializer = rmp_serde::Deserializer::new(&diff_bytes[..]);

    // Apply the diff to the current state
    if let Err(e) = Apply::apply(&mut deserializer, &mut current_state) {
        return Err(format!("Failed to apply diff: {}", e));
    }

    // Update the global state with the new state
    apply_entire_state(current_state);

    // Update timestamp and checksum
    update_checksum_for_state_diff(diff_data.clone());

    Ok(())
}

pub fn apply_entire_state(state: EntireState) {
    // About (ignores any state that doesnt actually change)
    OWNER_ID.with(|store| {
        *store.borrow_mut() = state.OWNER_ID;
    });
    URL_ENDPOINT.with(|store| {
        *store.borrow_mut() = state.URL_ENDPOINT;
    });
    
    // Api Keys
    APIKEYS_BY_VALUE_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.APIKEYS_BY_VALUE_HASHTABLE;
    });
    APIKEYS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.APIKEYS_BY_ID_HASHTABLE;
    });
    USERS_APIKEYS_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.USERS_APIKEYS_HASHTABLE;
    });
    
    // Contacts
    CONTACTS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.CONTACTS_BY_ID_HASHTABLE;
    });
    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE;
    });
    CONTACTS_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.CONTACTS_BY_TIME_LIST;
    });
    
    // Directory
    folder_uuid_to_metadata.with_mut(|map| {
        *map = state.folder_uuid_to_metadata;
    });
    file_uuid_to_metadata.with_mut(|map| {
        *map = state.file_uuid_to_metadata;
    });
    full_folder_path_to_uuid.with_mut(|map| {
        *map = state.full_folder_path_to_uuid;
    });
    full_file_path_to_uuid.with_mut(|map| {
        *map = state.full_file_path_to_uuid;
    });
    
    // Disks
    DISKS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.DISKS_BY_ID_HASHTABLE;
    });
    DISKS_BY_EXTERNAL_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.DISKS_BY_EXTERNAL_ID_HASHTABLE;
    });
    DISKS_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.DISKS_BY_TIME_LIST;
    });
    
    // Drives
    DRIVES_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.DRIVES_BY_ID_HASHTABLE;
    });
    DRIVES_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.DRIVES_BY_TIME_LIST;
    });
    
    // Permissions
    DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE;
    });
    DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE;
    });
    DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE;
    });
    DIRECTORY_PERMISSIONS_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.DIRECTORY_PERMISSIONS_BY_TIME_LIST;
    });
    SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.SYSTEM_PERMISSIONS_BY_ID_HASHTABLE;
    });
    SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE;
    });
    SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE;
    });
    SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.SYSTEM_PERMISSIONS_BY_TIME_LIST;
    });
    
    // Team Invites
    INVITES_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.INVITES_BY_ID_HASHTABLE;
    });
    USERS_INVITES_LIST_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.USERS_INVITES_LIST_HASHTABLE;
    });
    
    // Teams
    TEAMS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.TEAMS_BY_ID_HASHTABLE;
    });
    TEAMS_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.TEAMS_BY_TIME_LIST;
    });
    
    // Webhooks
    WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.WEBHOOKS_BY_ALT_INDEX_HASHTABLE;
    });
    WEBHOOKS_BY_ID_HASHTABLE.with(|store| {
        *store.borrow_mut() = state.WEBHOOKS_BY_ID_HASHTABLE;
    });
    WEBHOOKS_BY_TIME_LIST.with(|store| {
        *store.borrow_mut() = state.WEBHOOKS_BY_TIME_LIST;
    });
}