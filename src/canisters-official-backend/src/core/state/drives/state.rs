
// src/core/state/drives/state.rs

pub mod state {
    use std::cell::RefCell;
    use ic_stable_structures::memory_manager::MemoryId;
    use ic_stable_structures::StableBTreeMap;
    use ic_stable_structures::StableVec;
    use ic_stable_structures::StableCell;
    use ic_stable_structures::DefaultMemoryImpl;

    use crate::core::api::helpers::get_appropriate_url_endpoint;
    use crate::core::api::replay::diff::update_checksum_for_state_diff;
    use crate::core::api::uuid::format_drive_id;
    use crate::core::api::uuid::generate_uuidv4;
    use crate::core::state::contacts::state::state::CONTACTS_BY_TIME_LIST;
    use crate::core::state::drives::types::Drive;
    use crate::core::state::drives::types::DriveID;
    use crate::core::state::drives::types::DriveRESTUrlEndpoint;
    use crate::core::state::drives::types::ExternalID;
    use crate::core::state::drives::types::FactorySpawnHistoryRecord;
    use crate::core::state::drives::types::SpawnRedeemCode;
    use crate::core::state::drives::types::StateChecksum;
    use crate::core::state::drives::types::DriveStateDiffString;
    use crate::core::state::drives::types::StringVec;
    use crate::core::state::group_invites::state::state::INVITES_BY_ID_HASHTABLE;
    use crate::core::state::group_invites::types::GroupInviteeID;
    use crate::core::state::groups::state::state::GROUPS_BY_ID_HASHTABLE;
    use crate::core::state::groups::types::GroupID;
    use crate::core::state::permissions::types::PermissionGranteeID;
    use crate::core::state::webhooks::state::state::WEBHOOKS_BY_ID_HASHTABLE;
    use crate::core::state::webhooks::types::WebhookAltIndexID;
    use crate::core::types::ICPPrincipalString;
    use crate::core::types::IDPrefix;
    use crate::core::types::PublicKeyEVM;
    use crate::core::types::{UserID,PublicKeyICP};
    use crate::debug_log;
    use crate::MEMORY_MANAGER;

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

    pub const VERSION_MEMORY_ID: MemoryId = MemoryId::new(13);
    pub const DRIVE_STATE_CHECKSUM_MEMORY_ID: MemoryId = MemoryId::new(14);
    pub const DRIVE_STATE_TIMESTAMP_MEMORY_ID: MemoryId = MemoryId::new(15);
    pub const OWNER_ID_MEMORY_ID: MemoryId = MemoryId::new(16);
    pub const URL_ENDPOINT_MEMORY_ID: MemoryId = MemoryId::new(17);
    pub const TRANSFER_OWNER_ID_MEMORY_ID: MemoryId = MemoryId::new(18);
    pub const SPAWN_REDEEM_CODE_MEMORY_ID: MemoryId = MemoryId::new(19);
    pub const SPAWN_NOTE_MEMORY_ID: MemoryId = MemoryId::new(20);
    pub const NONCE_UUID_GENERATED_MEMORY_ID: MemoryId = MemoryId::new(21);
    pub const RECENT_DEPLOYMENTS_MEMORY_ID: MemoryId = MemoryId::new(22);
    pub const DRIVES_MEMORY_ID: MemoryId = MemoryId::new(23);
    pub const DRIVES_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(24);
    pub const EXTERNAL_ID_MAPPINGS_MEMORY_ID: MemoryId = MemoryId::new(25);
    pub const UUID_CLAIMED_MEMORY_ID: MemoryId = MemoryId::new(26);
    pub const NONCE_UUID_MEMORY_ID: MemoryId = MemoryId::new(27);
    

    thread_local! { 
        // self info - immutable
        pub(crate) static DRIVE_ID: DriveID = format_drive_id(&ic_cdk::api::id().to_text());
        pub(crate) static CANISTER_ID: PublicKeyICP = PublicKeyICP(ic_cdk::api::id().to_text());

        // Convert configuration values to stable cells
        pub(crate) static VERSION: RefCell<StableCell<String, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(VERSION_MEMORY_ID)),
                "OfficeX.Beta.0.0.1".to_string()
            ).expect("Failed to initialize VERSION")
        );
        
        pub(crate) static DRIVE_STATE_CHECKSUM: RefCell<StableCell<StateChecksum, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DRIVE_STATE_CHECKSUM_MEMORY_ID)),
                StateChecksum("genesis".to_string())
            ).expect("Failed to initialize DRIVE_STATE_CHECKSUM")
        );
        
        pub(crate) static DRIVE_STATE_TIMESTAMP_NS: RefCell<StableCell<u64, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DRIVE_STATE_TIMESTAMP_MEMORY_ID)),
                ic_cdk::api::time()
            ).expect("Failed to initialize DRIVE_STATE_TIMESTAMP_NS")
        );
        // Convert important user-related settings to stable cells
        pub(crate) static OWNER_ID: RefCell<StableCell<UserID, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(OWNER_ID_MEMORY_ID)),
                UserID("Anonymous_Owner".to_string())
            ).expect("Failed to initialize OWNER_ID")
        );
        
        pub(crate) static URL_ENDPOINT: RefCell<StableCell<DriveRESTUrlEndpoint, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(URL_ENDPOINT_MEMORY_ID)),
                DriveRESTUrlEndpoint(format!("https://{}.icp0.io", CANISTER_ID.with(|id| id.0.clone())))
            ).expect("Failed to initialize URL_ENDPOINT")
        );
        
        pub(crate) static TRANSFER_OWNER_ID: RefCell<StableCell<UserID, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(TRANSFER_OWNER_ID_MEMORY_ID)),
                UserID("".to_string())
            ).expect("Failed to initialize TRANSFER_OWNER_ID")
        );
        // Convert HashMap to StableBTreeMap for drives by ID
        pub(crate) static DRIVES_BY_ID_HASHTABLE: RefCell<StableBTreeMap<DriveID, Drive, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DRIVES_MEMORY_ID))
            )
        );
        
        // Convert Vec to StableVec for drives by time
        pub(crate) static DRIVES_BY_TIME_LIST: RefCell<StableVec<DriveID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DRIVES_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize DRIVES_BY_TIME_LIST")
        );
        
        // Convert simple counter to StableCell
        pub(crate) static NONCE_UUID_GENERATED: RefCell<StableCell<u128, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(NONCE_UUID_MEMORY_ID)),
                0
            ).expect("Failed to initialize NONCE_UUID_GENERATED")
        );
        
        // Convert HashMap to StableBTreeMap for external ID mappings
        pub(crate) static EXTERNAL_ID_MAPPINGS: RefCell<StableBTreeMap<ExternalID, StringVec, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(EXTERNAL_ID_MAPPINGS_MEMORY_ID))
            )
        );
        
        // Convert HashMap to StableBTreeMap for UUID claims
        pub(crate) static UUID_CLAIMED: RefCell<StableBTreeMap<String, bool, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(UUID_CLAIMED_MEMORY_ID))
            )
        );
        
        // Convert Vec to StableVec for deployment history
        pub(crate) static RECENT_DEPLOYMENTS: RefCell<StableVec<FactorySpawnHistoryRecord, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(RECENT_DEPLOYMENTS_MEMORY_ID))
            ).expect("Failed to initialize RECENT_DEPLOYMENTS")
        );
        
        // Convert String to StableCell
        pub(crate) static SPAWN_NOTE: RefCell<StableCell<String, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(SPAWN_NOTE_MEMORY_ID)),
                "".to_string()
            ).expect("Failed to initialize SPAWN_NOTE")
        );
        
        // Convert SpawnRedeemCode to StableCell
        pub(crate) static SPAWN_REDEEM_CODE: RefCell<StableCell<SpawnRedeemCode, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(SPAWN_REDEEM_CODE_MEMORY_ID)),
                SpawnRedeemCode("".to_string())
            ).expect("Failed to initialize SPAWN_REDEEM_CODE")
        );
    }


    pub fn initialize() {
        // Force thread_locals in this module to initialize
        VERSION.with(|_| {});
        DRIVE_STATE_CHECKSUM.with(|_| {});
        DRIVE_STATE_TIMESTAMP_NS.with(|_| {});
        OWNER_ID.with(|_| {});
        URL_ENDPOINT.with(|_| {});
        TRANSFER_OWNER_ID.with(|_| {});
        DRIVES_BY_ID_HASHTABLE.with(|_| {});
        DRIVES_BY_TIME_LIST.with(|_| {});
        NONCE_UUID_GENERATED.with(|_| {});
        EXTERNAL_ID_MAPPINGS.with(|_| {});
        UUID_CLAIMED.with(|_| {});
        RECENT_DEPLOYMENTS.with(|_| {});
        SPAWN_NOTE.with(|_| {});
        SPAWN_REDEEM_CODE.with(|_| {});
    }

    pub fn init_self_drive(
        owner_id: UserID,
        title: Option<String>,
        spawn_redeem_code: Option<String>,
        note: Option<String>,
    ) {
        debug_log!("Setting owner_id: {}", owner_id.0);
        OWNER_ID.with(|id| {
            id.borrow_mut().set(owner_id.clone());
            debug_log!("Confirmed owner_id set to: {}", id.borrow().get().0);
        });
        
        // Set spawn redeem code if provided
        let code = spawn_redeem_code.unwrap_or_else(|| "DEFAULT_SPAWN_REDEEM_CODE".to_string());
        debug_log!("Setting spawn redeem code to: {}", code);
        SPAWN_REDEEM_CODE.with(|c| {
            c.borrow_mut().set(SpawnRedeemCode(code.clone()));
            debug_log!("Confirmed spawn redeem code set to: {}", c.borrow().get().0);
        });
        
        // Set spawn note if provided
        let note = note.unwrap_or_else(|| "".to_string());
        debug_log!("Setting spawn note to: {}", note);
        SPAWN_NOTE.with(|n| {
            n.borrow_mut().set(note.clone());
        });

        // Handle the URL endpoint
        let endpoint = get_appropriate_url_endpoint();
        debug_log!("Setting URL endpoint to: {}", endpoint);
        URL_ENDPOINT.with(|url| {
            url.borrow_mut().set(DriveRESTUrlEndpoint(endpoint));
            debug_log!("Confirmed URL endpoint set to: {}", url.borrow().get().0);
        });
        
        // Use provided nickname or default
        let drive_name = title.unwrap_or_else(|| "Anonymous Org".to_string());
        debug_log!("Initializing self drive with name: {}", drive_name);
        
        let self_drive = Drive {
            id: DRIVE_ID.with(|id| id.clone()),
            name: drive_name,
            public_note: Some("".to_string()),
            private_note: Some("".to_string()),
            icp_principal: ICPPrincipalString(PublicKeyICP(ic_cdk::api::id().to_text())),
            host_url: URL_ENDPOINT.with(|url| url.borrow().get().clone()),
            last_indexed_ms: None,
            labels: vec![],
            external_id: None,
            external_payload: None,
            created_at: ic_cdk::api::time() / 1_000_000,
        };

        DRIVES_BY_ID_HASHTABLE.with(|map| {
            map.borrow_mut().insert(self_drive.id.clone(), self_drive.clone());
        });

        DRIVES_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(&self_drive.id);
        });

        update_checksum_for_state_diff(DriveStateDiffString("".to_string()));
    }

    pub fn update_external_id_mapping(
        old_external_id: Option<ExternalID>,
        new_external_id: Option<ExternalID>,
        internal_id: Option<String>,
    ) {
        if internal_id.is_none() {
            // Can't do anything without internal_id; safely return early
            return;
        }
        EXTERNAL_ID_MAPPINGS.with(|mappings| {
            let mut mappings_mut = mappings.borrow_mut();
            
            // Handle removal of old external ID mapping if it exists
            if let Some(old_id) = old_external_id {
                if let Some(ids) = mappings_mut.get(&old_id) {
                    let mut ids_clone = ids.clone();
                    // Remove the internal_id from the old mapping
                    ids_clone.retain(|id| id != internal_id.as_ref().unwrap());
                    
                    // If the vector is now empty, remove the mapping entirely
                    if ids_clone.is_empty() {
                        mappings_mut.remove(&old_id);
                    } else {
                        mappings_mut.insert(old_id, ids_clone);
                    }
                }
            }
            
            // Handle adding new external ID mapping if it exists
            let internal_id = internal_id.unwrap();
            if let Some(new_id) = new_external_id {
                let mut ids = mappings_mut.get(&new_id).unwrap_or_else(|| StringVec { items: vec![] }).clone();
                if !ids.items.contains(&internal_id) {
                    ids.items.push(internal_id.clone());
                }
                mappings_mut.insert(new_id, ids);
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
                // Update the contacts icp principal
                contact.icp_principal = new_user_id.to_icp_principal_string();
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

        CONTACTS_BY_TIME_LIST.with(|store| {
            let mut time_list = store.borrow_mut();
            for i in 0..time_list.len() {
                // get() returns a cloned value, not a reference, so no need to dereference
                if let Some(user_id) = time_list.get(i) {
                    if user_id == old_user_id {
                        // set() expects a reference, so pass &new_user_id
                        time_list.set(i, &new_user_id);
                    }
                }
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
            
            if let Some(permission_ids) = map.get(&old_grantee) {
                // Clone the permission IDs
                let permission_ids_clone = permission_ids.clone();
                
                // Remove from old user and add to new user
                map.remove(&old_grantee);
                map.insert(new_grantee.clone(), permission_ids_clone.clone());
                
                // Update the individual permissions
                crate::core::state::permissions::state::state::DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|perms| {
                    let mut perms_mut = perms.borrow_mut();
                    
                    // Get all keys in the hashtable
                    let all_keys: Vec<crate::core::state::permissions::types::DirectoryPermissionID> = 
                        perms_mut.iter().map(|(k, _)| k.clone()).collect();
                    
                    // Process each permission
                    for key in all_keys {
                        if let Some(perm) = perms_mut.get(&key) {
                            let mut updated_perm = perm.clone();
                            let mut modified = false;
                            
                            // Check if granted_to is a User type and matches old_user_id
                            if let crate::core::state::permissions::types::PermissionGranteeID::User(user_id) = &updated_perm.granted_to {
                                if *user_id == old_user_id {
                                    updated_perm.granted_to = crate::core::state::permissions::types::PermissionGranteeID::User(new_user_id.clone());
                                    modified = true;
                                    count += 1;
                                }
                            }
                            
                            // If we made changes, insert the modified permission back
                            if modified {
                                perms_mut.insert(key, updated_perm);
                            }
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
            
            if let Some(permission_ids) = map.get(&old_grantee) {
                // Clone the permission IDs
                let permission_ids_clone = permission_ids.clone();
                
                // Remove from old user and add to new user
                map.remove(&old_grantee);
                map.insert(new_grantee.clone(), permission_ids_clone.clone());
                
                // Update the individual permissions
                crate::core::state::permissions::state::state::SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|perms| {
                    let mut perms_mut = perms.borrow_mut();
                    for permission_id in &permission_ids_clone.permissions {
                        if let Some(perm) = perms_mut.get(permission_id) {
                            let mut updated_perm = perm.clone();
                            
                            // Check and update granted_by
                            if updated_perm.granted_by == old_user_id {
                                updated_perm.granted_by = new_user_id.clone();
                                count += 1;
                            }
                            
                            if let PermissionGranteeID::User(user_id) = &updated_perm.granted_to {
                                if *user_id == old_user_id {
                                    let new_grantee = PermissionGranteeID::User(new_user_id.clone());
                                    updated_perm.granted_to = new_grantee;
                                    count += 1;
                                }
                            }
                            
                            perms_mut.insert(permission_id.clone(), updated_perm);
                        }
                    }
                });
                
                count += 1; // Count the hashtable entry update
            }
            
            count
        });
    
        // 8. Update USERS_INVITES_LIST_HASHTABLE - optimize by using invite IDs
        update_count += crate::core::state::group_invites::state::state::USERS_INVITES_LIST_HASHTABLE.with(|map| {
            let mut map = map.borrow_mut();
            let mut count = 0;
            
            // Find and update invites where the user is the invitee
            let old_invitee = crate::core::state::group_invites::types::GroupInviteeID::User(old_user_id.clone());
            let new_invitee = crate::core::state::group_invites::types::GroupInviteeID::User(new_user_id.clone());
            
            if let Some(invite_ids) = map.remove(&old_invitee) {
                // Associate invites with the new user ID
                map.insert(new_invitee.clone(), invite_ids.clone());
                
                // Update the individual invites
                INVITES_BY_ID_HASHTABLE.with(|invites| {
                    let mut invites = invites.borrow_mut();
                    for invite_id in &invite_ids.invites {
                        // Get and clone the invite
                        if let Some(mut invite) = invites.get(invite_id).clone() {
                            let mut modified = false;
                            
                            // Make modifications to the cloned value
                            if invite.inviter_id == old_user_id {
                                invite.inviter_id = new_user_id.clone();
                                modified = true;
                                count += 1;
                            }
                            
                            if invite.invitee_id == GroupInviteeID::User(old_user_id.clone()) {
                                invite.invitee_id = GroupInviteeID::User(new_user_id.clone());
                                modified = true;
                                count += 1;
                            }
                            
                            // If we made changes, insert the modified value back
                            if modified {
                                invites.insert(invite_id.clone(), invite);
                            }
                        }
                    }
                });
                
                count += 1; // Count the hashtable entry update
            }
            
            count
        });
    
        // 9. Update GROUPS_BY_TIME_LIST (Groups where user is the owner)
        update_count += GROUPS_BY_ID_HASHTABLE.with(|groups| {
            let mut groups = groups.borrow_mut();
            let mut count = 0;
            
            // First collect all the keys that need to be updated
            let all_keys: Vec<GroupID> = groups.iter().map(|(k, _)| k.clone()).collect();
            
            // Then process each entry
            for key in all_keys {
                if let Some(mut group) = groups.get(&key).clone() {
                    if group.owner == old_user_id {
                        group.owner = new_user_id.clone();
                        groups.insert(key, group);
                        count += 1;
                    }
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
                WEBHOOKS_BY_ID_HASHTABLE.with(|webhooks_map| {
                    let mut webhooks_map = webhooks_map.borrow_mut();
                    
                    for webhook_id in webhook_ids.webhooks {
                        // Get the webhook, modify it, and insert it back
                        if let Some(mut webhook) = webhooks_map.get(&webhook_id).clone() {
                            webhook.alt_index = WebhookAltIndexID(new_user_id.0.clone());
                            webhooks_map.insert(webhook_id, webhook);
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


