
// src/core/state/drives/state.rs

pub mod state {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::api::replay::diff::update_checksum_for_state_diff;
    use crate::core::api::uuid::format_drive_id;
    use crate::core::api::uuid::generate_unique_id;
    use crate::core::state::drives::types::Drive;
    use crate::core::state::drives::types::DriveID;
    use crate::core::state::drives::types::DriveRESTUrlEndpoint;
    use crate::core::state::drives::types::ExternalID;
    use crate::core::state::drives::types::StateChecksum;
    use crate::core::state::drives::types::DriveStateDiffString;
    use crate::core::state::team_invites::types::TeamInviteeID;
    use crate::core::state::webhooks::types::WebhookAltIndexID;
    use crate::core::types::ICPPrincipalString;
    use crate::core::types::IDPrefix;
    use crate::core::types::{UserID,PublicKeyICP};
    use crate::debug_log;

    thread_local! { 
        // self info - immutable
        pub(crate) static DRIVE_ID: DriveID = format_drive_id(&ic_cdk::api::id().to_text());
        pub(crate) static CANISTER_ID: PublicKeyICP = PublicKeyICP(ic_cdk::api::id().to_text());
        pub(crate) static GLOBAL_UUID_NONCE: Cell<u64> = Cell::new(0);
        pub(crate) static DRIVE_STATE_CHECKSUM: RefCell<StateChecksum> = RefCell::new(StateChecksum("genesis".to_string()));
        pub(crate) static DRIVE_STATE_TIMESTAMP_NS: Cell<u64> = Cell::new(ic_cdk::api::time());
        // self info - mutable
        pub(crate) static OWNER_ID: RefCell<UserID> = RefCell::new(UserID("Anonymous_Owner".to_string()));
        pub(crate) static URL_ENDPOINT: RefCell<DriveRESTUrlEndpoint> = RefCell::new(DriveRESTUrlEndpoint(format!("https://{}.icp0.io", CANISTER_ID.with(|id| id.0.clone()))));
        pub(crate) static TRANSFER_OWNER_ID: RefCell<UserID> = RefCell::new(UserID("".to_string()));
        // hashtables
        pub(crate) static DRIVES_BY_ID_HASHTABLE: RefCell<HashMap<DriveID, Drive>> = RefCell::new(HashMap::new());
        pub(crate) static DRIVES_BY_TIME_LIST: RefCell<Vec<DriveID>> = RefCell::new(Vec::new());
        // external id tracking
        pub(crate) static EXTERNAL_ID_MAPPINGS: RefCell<HashMap<ExternalID, Vec<String>>> = RefCell::new(HashMap::new());
    }

    pub fn init_self_drive() {
        let self_drive = Drive {
            id: DRIVE_ID.with(|id| id.clone()),
            name: "Anonymous_Canister".to_string(),
            public_note: Some("".to_string()),
            private_note: Some("".to_string()),
            icp_principal: ICPPrincipalString(PublicKeyICP(ic_cdk::api::id().to_text())),
            url_endpoint: URL_ENDPOINT.with(|url| url.borrow().clone()),
            last_indexed_ms: None,
            tags: vec![],
            external_id: None,
            external_payload: None,
        };

        DRIVES_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(self_drive.id.clone(), self_drive.clone());
        });

        DRIVES_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(self_drive.id.clone());
        });

        update_checksum_for_state_diff(DriveStateDiffString("".to_string()));
    }

    pub fn update_external_id_mapping(
        old_external_id: Option<ExternalID>,
        new_external_id: Option<ExternalID>,
        internal_id: Option<String>,
    ) {
        EXTERNAL_ID_MAPPINGS.with(|mappings| {
            let mut mappings_mut = mappings.borrow_mut();
            
            // Handle removal of old external ID mapping if it exists
            if let Some(old_id) = old_external_id {
                if let Some(ids) = mappings_mut.get_mut(&old_id) {
                    // Remove the internal_id from the old mapping
                    ids.retain(|id| id != internal_id.as_ref().unwrap());
                    
                    // If the vector is now empty, remove the mapping entirely
                    if ids.is_empty() {
                        mappings_mut.remove(&old_id);
                    }
                }
            }
            
            // Handle adding new external ID mapping if it exists
            let internal_id = internal_id.unwrap();
            if let Some(new_id) = new_external_id {
                mappings_mut
                    .entry(new_id)
                    .and_modify(|ids| {
                        // Only add if it's not already in the list
                        if !ids.contains(&internal_id) {
                            ids.push(internal_id.clone());
                        }
                    })
                    .or_insert_with(|| vec![internal_id.clone()]);
            }
        });
        
    }

    pub fn superswap_userid(
        old_user_id: UserID,
        new_user_id: UserID,
    ) -> Result<i32, String> {
        debug_log!("Performing user ID superswap from {} to {}", old_user_id, new_user_id);
        let mut update_count = 0;
    
        // 1. Update USERS_APIKEYS_HASHTABLE
        update_count += crate::core::state::api_keys::state::state::USERS_APIKEYS_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            // Get the API keys associated with the old user ID
            if let Some(api_keys) = map.remove(&old_user_id) {
                // Associate them with the new user ID
                map.insert(new_user_id.clone(), api_keys);
                1
            } else {
                0
            }
        });
    
        // 2. Update CONTACTS_BY_ID_HASHTABLE
        update_count += crate::core::state::contacts::state::state::CONTACTS_BY_ID_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            // Check if the old UserID exists as a contact
            if let Some(mut contact) = map.remove(&old_user_id) {
                // Update the contact's ID
                contact.id = new_user_id.clone();
                // Add to past_user_ids if not already there
                if !contact.past_user_ids.contains(&old_user_id) {
                    contact.past_user_ids.push(old_user_id.clone());
                }
                // Re-insert with new ID
                map.insert(new_user_id.clone(), contact);
                1
            } else {
                0
            }
        });
    
        // 3. Update CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE
        let old_icp_principal = old_user_id.to_icp_principal_string();
        let new_icp_principal = new_user_id.to_icp_principal_string();
        update_count += crate::core::state::contacts::state::state::CONTACTS_BY_ICP_PRINCIPAL_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            if let Some(_) = map.remove(&old_icp_principal) {
                map.insert(new_icp_principal.clone(), new_user_id.clone());
                1
            } else {
                0
            }
        });
    
        // 4. Update HISTORY_SUPERSWAP_USERID
        crate::core::state::contacts::state::state::HISTORY_SUPERSWAP_USERID.with(|map| {
            let mut map = map.borrow_mut();
            // Record the superswap in history (new format is HashMap<UserID, UserID>)
            map.insert(old_user_id.clone(), new_user_id.clone());
        });
    
        // Skip directory file and folder updates as unnecessary
    
        // 6. Update Directory Permissions - optimize by using permission IDs
        update_count += crate::core::state::permissions::state::state::DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            let mut count = 0;
            
            // Get permissions associated with the old user ID
            let old_grantee = crate::core::state::permissions::types::PermissionGranteeID::User(old_user_id.clone());
            let new_grantee = crate::core::state::permissions::types::PermissionGranteeID::User(new_user_id.clone());
            
            if let Some(permission_ids) = map.remove(&old_grantee) {
                // Associate them with the new user ID
                map.insert(new_grantee.clone(), permission_ids.clone());
                
                // Update the individual permissions
                crate::core::state::permissions::state::state::DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|perms| {
                    let mut perms = perms.borrow_mut();
                    for permission_id in &permission_ids {
                        if let Some(perm) = perms.get_mut(permission_id) {
                            if perm.granted_by == old_user_id {
                                perm.granted_by = new_user_id.clone();
                                count += 1;
                            }
                            
                            // The granted_to is already updated by moving the entry in the hashtable
                        }
                    }
                });
                
                count += 1; // Count the hashtable entry update
            }
            
            count
        });
    
        // 7. Update System Permissions - optimize by using permission IDs
        update_count += crate::core::state::permissions::state::state::SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            let mut count = 0;
            
            // Get permissions associated with the old user ID
            let old_grantee = crate::core::state::permissions::types::PermissionGranteeID::User(old_user_id.clone());
            let new_grantee = crate::core::state::permissions::types::PermissionGranteeID::User(new_user_id.clone());
            
            if let Some(permission_ids) = map.remove(&old_grantee) {
                // Associate them with the new user ID
                map.insert(new_grantee.clone(), permission_ids.clone());
                
                // Update the individual permissions
                crate::core::state::permissions::state::state::SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|perms| {
                    let mut perms = perms.borrow_mut();
                    for permission_id in &permission_ids {
                        if let Some(perm) = perms.get_mut(permission_id) {
                            if perm.granted_by == old_user_id {
                                perm.granted_by = new_user_id.clone();
                                count += 1;
                            }
                            
                            // The granted_to is already updated by moving the entry in the hashtable
                        }
                    }
                });
                
                count += 1; // Count the hashtable entry update
            }
            
            count
        });
    
        // 8. Update USERS_INVITES_LIST_HASHTABLE - optimize by using invite IDs
        update_count += crate::core::state::team_invites::state::state::USERS_INVITES_LIST_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            let mut count = 0;
            
            // Find and update invites where the user is the invitee
            let old_invitee = crate::core::state::team_invites::types::TeamInviteeID::User(old_user_id.clone());
            let new_invitee = crate::core::state::team_invites::types::TeamInviteeID::User(new_user_id.clone());
            
            if let Some(invite_ids) = map.remove(&old_invitee) {
                // Associate invites with the new user ID
                map.insert(new_invitee.clone(), invite_ids.clone());
                
                // Update the individual invites
                crate::core::state::team_invites::state::state::INVITES_BY_ID_HASHTABLE.with(|invites| {
                    let mut invites = invites.borrow_mut();
                    for invite_id in &invite_ids {
                        if let Some(invite) = invites.get_mut(invite_id) {
                            if invite.inviter_id == old_user_id {
                                invite.inviter_id = new_user_id.clone();
                                count += 1;
                            }
                            if invite.invitee_id == TeamInviteeID::User(old_user_id.clone()) {
                                invite.invitee_id = TeamInviteeID::User(new_user_id.clone());
                                count += 1;
                            }
                        }
                    }
                });
                
                count += 1; // Count the hashtable entry update
            }
            
            count
        });
    
        // 9. Update TEAMS_BY_TIME_LIST (Teams where user is the owner)
        update_count += crate::core::state::teams::state::state::TEAMS_BY_ID_HASHTABLE.with(|teams| {
            let mut teams = teams.borrow_mut();
            let mut count = 0;
            
            for team in teams.values_mut() {
                if team.owner == old_user_id {
                    team.owner = new_user_id.clone();
                    count += 1;
                }
            }
            
            count
        });
    
        // 10. Update WEBHOOKS_BY_ALT_INDEX_HASHTABLE
        update_count += crate::core::state::webhooks::state::state::WEBHOOKS_BY_ALT_INDEX_HASHTABLE.with(|alt_index_map| {
            let mut alt_index_map = alt_index_map.borrow_mut();
            let mut count = 0;
            
            // Get webhooks associated with the old user ID
            let user_key = WebhookAltIndexID(old_user_id.0.clone());
            if let Some(webhook_ids) = alt_index_map.remove(&user_key) {
                // Add webhooks to the new user ID
                alt_index_map.insert(WebhookAltIndexID(new_user_id.0.clone()), webhook_ids.clone());
                
                // Update the webhook objects themselves
                crate::core::state::webhooks::state::state::WEBHOOKS_BY_ID_HASHTABLE.with(|webhooks_map| {
                    let mut webhooks_map = webhooks_map.borrow_mut();
                    
                    for webhook_id in webhook_ids {
                        if let Some(webhook) = webhooks_map.get_mut(&webhook_id) {
                            webhook.alt_index = WebhookAltIndexID(new_user_id.0.clone());
                            count += 1;
                        }
                    }
                });
                
                count += 1; // Count the hashtable update itself
            }
            
            count // Return the count regardless of if/else path
        });
    
        debug_log!("User ID superswap completed. Updated {} references.", update_count);
        Ok(update_count)
    }
}


