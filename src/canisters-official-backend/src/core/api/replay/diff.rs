// src/core/api/replay/diff.rs

use serde_diff::Apply;
use serde_diff::{Diff, SerdeDiff};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use crate::core::state::contacts::state::state::HISTORY_SUPERSWAP_USERID;
use crate::core::state::drives::state::state::{DRIVE_STATE_CHECKSUM, EXTERNAL_ID_MAPPINGS, RECENT_DEPLOYMENTS, SPAWN_NOTE, SPAWN_REDEEM_CODE};
use crate::core::state::drives::types::{DriveStateDiffID, ExternalID, FactorySpawnHistoryRecord, SpawnRedeemCode, StateChecksum, StateDiffRecord};
use crate::core::types::{ICPPrincipalString, PublicKeyEVM};
use crate::{core::{api::{webhooks::state_diffs::{fire_state_diff_webhooks, get_active_state_diff_webhooks}}, state::{api_keys::{state::state::{APIKEYS_BY_ID_HASHTABLE, APIKEYS_BY_VALUE_HASHTABLE, USERS_APIKEYS_HASHTABLE}, types::{ApiKey, ApiKeyID, ApiKeyValue}}, contacts::{state::state::{CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE, CONTACTS_BY_ID_HASHTABLE, CONTACTS_BY_TIME_LIST}, types::Contact}, directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FileRecord, FileID, FolderRecord, FolderID}}, disks::{state::state::{DISKS_BY_ID_HASHTABLE, DISKS_BY_TIME_LIST}, types::{Disk, DiskID}}, drives::{state::state::{CANISTER_ID, DRIVES_BY_ID_HASHTABLE, DRIVES_BY_TIME_LIST, DRIVE_ID, DRIVE_STATE_TIMESTAMP_NS, OWNER_ID, URL_ENDPOINT}, types::{Drive, DriveID, DriveRESTUrlEndpoint, DriveStateDiffString}}, permissions::{state::state::{DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE, DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE, DIRECTORY_PERMISSIONS_BY_TIME_LIST, SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE, SYSTEM_PERMISSIONS_BY_TIME_LIST}, types::{DirectoryPermission, DirectoryPermissionID, PermissionGranteeID, SystemPermission, SystemPermissionID, SystemResourceID}}, team_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::{TeamInviteID, TeamInviteeID, Team_Invite}}, teams::{state::state::{TEAMS_BY_ID_HASHTABLE, TEAMS_BY_TIME_LIST}, types::{Team, TeamID}}, webhooks::{state::state::{WEBHOOKS_BY_ALT_INDEX_HASHTABLE, WEBHOOKS_BY_ID_HASHTABLE, WEBHOOKS_BY_TIME_LIST}, types::{Webhook, WebhookAltIndexID, WebhookID}}}, types::{PublicKeyICP, UserID}}, rest::directory::types::DirectoryResourceID};

// Define a type to represent the entire state
#[derive(SerdeDiff, Serialize, Deserialize, Clone, Debug)]
pub struct EntireState {
    // About
    DRIVE_ID: DriveID,
    CANISTER_ID: PublicKeyICP,
    OWNER_ID: UserID,
    URL_ENDPOINT: DriveRESTUrlEndpoint,
    DRIVE_STATE_TIMESTAMP_NS: u64,
    EXTERNAL_ID_MAPPINGS: HashMap<ExternalID, Vec<String>>,
    RECENT_DEPLOYMENTS: Vec<FactorySpawnHistoryRecord>,
    SPAWN_REDEEM_CODE: SpawnRedeemCode,
    SPAWN_NOTE: String,
    // Api Keys
    APIKEYS_BY_VALUE_HASHTABLE: HashMap<ApiKeyValue, ApiKeyID>,
    APIKEYS_BY_ID_HASHTABLE: HashMap<ApiKeyID, ApiKey>,
    USERS_APIKEYS_HASHTABLE: HashMap<UserID, Vec<ApiKeyID>>,
    // Contacts
    CONTACTS_BY_ID_HASHTABLE: HashMap<UserID, Contact>,
    CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: HashMap<ICPPrincipalString, UserID>,
    CONTACTS_BY_TIME_LIST: Vec<UserID>,
    HISTORY_SUPERSWAP_USERID: HashMap<UserID, UserID>,
    // Directory
    folder_uuid_to_metadata: HashMap<FolderID, FolderRecord>,
    file_uuid_to_metadata: HashMap<FileID, FileRecord>,
    full_folder_path_to_uuid: HashMap<DriveFullFilePath, FolderID>,
    full_file_path_to_uuid: HashMap<DriveFullFilePath, FileID>,
    // Disks
    DISKS_BY_ID_HASHTABLE: HashMap<DiskID, Disk>,
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

pub fn snapshot_entire_state() -> EntireState {
    EntireState {
        // About
        DRIVE_ID: DRIVE_ID.with(|drive_id| drive_id.clone()),
        CANISTER_ID: CANISTER_ID.with(|canister_id| canister_id.clone()),
        OWNER_ID: OWNER_ID.with(|owner_id| owner_id.borrow().clone()),
        URL_ENDPOINT: URL_ENDPOINT.with(|url| url.borrow().clone()),
        DRIVE_STATE_TIMESTAMP_NS: DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get()),
        EXTERNAL_ID_MAPPINGS: EXTERNAL_ID_MAPPINGS.with(|store| store.borrow().clone()),
        RECENT_DEPLOYMENTS: RECENT_DEPLOYMENTS.with(|store| store.borrow().clone()),
        SPAWN_REDEEM_CODE: SPAWN_REDEEM_CODE.with(|store| store.borrow().clone()),
        SPAWN_NOTE: SPAWN_NOTE.with(|store| store.borrow().clone()),
        // Api Keys
        APIKEYS_BY_VALUE_HASHTABLE: APIKEYS_BY_VALUE_HASHTABLE.with(|store| store.borrow().clone()),
        APIKEYS_BY_ID_HASHTABLE: APIKEYS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        USERS_APIKEYS_HASHTABLE: USERS_APIKEYS_HASHTABLE.with(|store| store.borrow().clone()),
        // Contacts
        CONTACTS_BY_ID_HASHTABLE: CONTACTS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
        CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE: CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|store| store.borrow().clone()),
        CONTACTS_BY_TIME_LIST: CONTACTS_BY_TIME_LIST.with(|store| store.borrow().clone()),
        HISTORY_SUPERSWAP_USERID: HISTORY_SUPERSWAP_USERID.with(|store| store.borrow().clone()),
        // Directory
        folder_uuid_to_metadata: folder_uuid_to_metadata.with(|store| store.clone()),
        file_uuid_to_metadata: file_uuid_to_metadata.with(|store| store.clone()),
        full_folder_path_to_uuid: full_folder_path_to_uuid.with(|store| store.clone()),
        full_file_path_to_uuid: full_file_path_to_uuid.with(|store| store.clone()),
        // Disks
        DISKS_BY_ID_HASHTABLE: DISKS_BY_ID_HASHTABLE.with(|store| store.borrow().clone()),
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

pub fn calculate_new_checksum(prev_checksum: &StateChecksum, diff_string: &DriveStateDiffString) -> StateChecksum {
    let input = format!("{}:{}", prev_checksum.0, diff_string.0);
    StateChecksum(mock_hash(&input))
}

pub fn snapshot_poststate(before_snapshot: Option<EntireState>, notes: Option<String>) {
    match before_snapshot {
        Some(before_snapshot) => {
            let after_snapshot = snapshot_entire_state();
            match diff_entire_state(before_snapshot, after_snapshot) {
                Some((forward_diff, backward_diff)) => {
                    // Calculate forward checksum
                    let prev_checksum = DRIVE_STATE_CHECKSUM.with(|cs| cs.borrow().clone());
                    let forward_checksum = calculate_new_checksum(&prev_checksum, &forward_diff);
                    
                    // Calculate backward checksum
                    let backward_checksum = calculate_new_checksum(&forward_checksum, &backward_diff);
                    
                    // Update current state checksum to forward checksum
                    DRIVE_STATE_CHECKSUM.with(|cs| {
                        *cs.borrow_mut() = forward_checksum.clone();
                    });
                    
                    // Update timestamp
                    DRIVE_STATE_TIMESTAMP_NS.with(|ts| {
                        ts.set(ic_cdk::api::time());
                    });
                    
                    fire_state_diff_webhooks(
                        forward_diff, 
                        backward_diff, 
                        forward_checksum,
                        backward_checksum,
                        notes
                    );
                },
                None => ()
            }
        },
        None => ()
    }
}

pub fn diff_entire_state(before_snapshot: EntireState, after_snapshot: EntireState) -> Option<(DriveStateDiffString, DriveStateDiffString)> {
    // Create MessagePack diff for forward direction (before -> after)
    let forward_diff_data = match rmp_serde::to_vec_named(&Diff::serializable(&before_snapshot, &after_snapshot)) {
        Ok(data) => data,
        Err(e) => {
            ic_cdk::println!("Failed to serialize forward state diff: {}", e);
            Vec::new()
        }
    };

    // Create MessagePack diff for backward direction (after -> before)
    let backward_diff_data = match rmp_serde::to_vec_named(&Diff::serializable(&after_snapshot, &before_snapshot)) {
        Ok(data) => data,
        Err(e) => {
            ic_cdk::println!("Failed to serialize backward state diff: {}", e);
            Vec::new()
        }
    };

    if forward_diff_data.len() <= 4 {  // Adjust this threshold based on testing
        return None;  // No meaningful difference, skip firing
    }
    
    // Convert diffs to base64 for transmission
    let forward_diff_base64 = base64::encode(&forward_diff_data);
    let backward_diff_base64 = base64::encode(&backward_diff_data);
    
    Some((
        DriveStateDiffString(forward_diff_base64),
        DriveStateDiffString(backward_diff_base64)
    ))
}

pub fn apply_state_diff(diff_data: &DriveStateDiffString, expected_checksum: &StateChecksum) -> Result<StateChecksum, String> {
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

    // Calculate new checksum
    let new_checksum = calculate_new_checksum(expected_checksum, diff_data);
    
    // Update stored checksum
    DRIVE_STATE_CHECKSUM.with(|cs| {
        *cs.borrow_mut() = new_checksum.clone();
    });
    
    // Update timestamp
    DRIVE_STATE_TIMESTAMP_NS.with(|ts| {
        ts.set(ic_cdk::api::time());
    });

    Ok(new_checksum)
}

pub fn apply_entire_state(state: EntireState) {
    // About (ignores any state that doesnt actually change)
    OWNER_ID.with(|store| {
        *store.borrow_mut() = state.OWNER_ID;
    });
    URL_ENDPOINT.with(|store| {
        *store.borrow_mut() = state.URL_ENDPOINT;
    });
    EXTERNAL_ID_MAPPINGS.with(|store| {
        *store.borrow_mut() = state.EXTERNAL_ID_MAPPINGS;
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

// Update checksum based on a diff
pub fn update_checksum_for_state_diff(diff_string: DriveStateDiffString) {
    // Get previous checksum
    let prev_checksum = DRIVE_STATE_CHECKSUM.with(|cs| cs.borrow().0.clone());
    
    // Input for hash includes previous checksum and new diff
    let input = format!("{}:{}", prev_checksum, diff_string);
    
    // Generate new checksum
    let new_checksum = mock_hash(&input);
    
    // Update stored checksum
    DRIVE_STATE_CHECKSUM.with(|cs| {
        *cs.borrow_mut() = StateChecksum(new_checksum);
    });
    
    // Update timestamp
    DRIVE_STATE_TIMESTAMP_NS.with(|ts| {
        ts.set(ic_cdk::api::time());
    });
}

pub fn mock_hash(input: &str) -> String {
    // Get the DRIVE_ID as salt
    let salt = DRIVE_ID.with(|id| id.0.clone());
    
    // Interweave characters from input and salt
    let mut result = String::with_capacity(64);
    let salt_chars: Vec<char> = salt.chars().collect();
    let input_chars: Vec<char> = input.chars().collect();
    let salt_len = salt_chars.len();
    let input_len = input_chars.len();
    
    for i in 0..64 {
        if i % 2 == 0 {
            // Even positions get input chars (if available)
            if i/2 < input_len {
                result.push(input_chars[i/2]);
            } else {
                result.push('0');
            }
        } else {
            // Odd positions get salt chars (if available)
            if i/2 < salt_len {
                result.push(salt_chars[i/2]);
            } else {
                result.push('1');
            }
        }
    }
    
    // Truncate or pad to exactly 64 chars
    if result.len() > 64 {
        result.truncate(64);
    } else {
        while result.len() < 64 {
            result.push('0');
        }
    }
    
    result
}


// Apply a sequence of diffs with validation and safety
pub fn safely_apply_diffs(diffs: &[StateDiffRecord]) -> Result<(usize, Option<DriveStateDiffID>), String> {
    if diffs.is_empty() {
        return Ok((0, None));
    }
    
    // Determine direction by checking timestamps
    let current_timestamp = DRIVE_STATE_TIMESTAMP_NS.with(|ts| ts.get());
    let is_reverse = diffs[0].timestamp_ns < current_timestamp;
    
    // Backup current state and checksum
    let backup_state = snapshot_entire_state();
    let original_checksum = DRIVE_STATE_CHECKSUM.with(|cs| cs.borrow().clone());
    
    // Sort diffs appropriately for the direction
    let mut sorted_diffs = diffs.to_vec();
    if is_reverse {
        // For reverse, sort by descending timestamp (newest to oldest)
        sorted_diffs.sort_by(|a, b| b.timestamp_ns.cmp(&a.timestamp_ns));
    } else {
        // For forward, sort by ascending timestamp (oldest to newest)
        sorted_diffs.sort_by(|a, b| a.timestamp_ns.cmp(&b.timestamp_ns));
    }
    
    // Apply diffs in sorted order
    let mut applied_count = 0;
    let mut last_diff_id = None;
    let mut current_checksum = original_checksum.clone();
    
    for diff in &sorted_diffs {
        // Select appropriate diff and expected checksum based on direction
        let (diff_to_apply, expected_checksum) = if is_reverse {
            (&diff.diff_backward, &diff.checksum_backward)
        } else {
            (&diff.diff_forward, &diff.checksum_forward)
        };
        
        // Validate checksum chain
        if applied_count > 0 && expected_checksum.0 != current_checksum.0 {
            // Chain validation failed - rollback
            apply_entire_state(backup_state);
            DRIVE_STATE_CHECKSUM.with(|cs| {
                *cs.borrow_mut() = original_checksum.clone();
            });
            
            return Err(format!(
                "Invalid checksum chain at diff {}. Expected: {}, Found: {}",
                diff.id, expected_checksum.0, current_checksum.0
            ));
        }
        
        // Apply the diff
        match apply_state_diff(diff_to_apply, &current_checksum) {
            Ok(new_checksum) => {
                applied_count += 1;
                last_diff_id = Some(diff.id.clone());
                current_checksum = new_checksum;
            },
            Err(e) => {
                // Application error - rollback
                apply_entire_state(backup_state);
                DRIVE_STATE_CHECKSUM.with(|cs| {
                    *cs.borrow_mut() = original_checksum.clone();
                });
                
                return Err(format!("Failed to apply diff {}: {}", diff.id, e));
            }
        }
    }
    
    Ok((applied_count, last_diff_id))
}


#[derive(Debug, Serialize, Deserialize)]
struct StateDiffChecksumShape {
    timestamp_ns: u64,
    diff_string: DriveStateDiffString,
}
