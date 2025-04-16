// src/core/state/groups/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
    use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap,StableCell,DefaultMemoryImpl, StableVec};
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;
    use crate::{core::{api::uuid::generate_uuidv4, state::{drives::state::state::{DRIVE_ID, OWNER_ID}, group_invites::{state::state::USERS_INVITES_LIST_HASHTABLE, types::{GroupInvite, GroupInviteIDList, GroupRole}}, permissions::{state::{helpers::{add_system_permission_to_grantee, add_system_permission_to_resource, update_system_permissions_time_list}, state::{SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE, SYSTEM_PERMISSIONS_BY_TIME_LIST}}, types::{PermissionGranteeID, SystemPermission, SystemPermissionID, SystemPermissionType, SystemResourceID, SystemTableEnum}}}, types::IDPrefix}, debug_log, rest::groups::types::ValidateGroupResponseData, MEMORY_MANAGER};
    use serde_json::json;

    use crate::core::{state::{drives::state::state::URL_ENDPOINT, group_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::{GroupInviteID, GroupInviteeID}}, groups::types::{Group, GroupID}}, types::UserID};
    
    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

    pub const GROUPS_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(31);
    pub const GROUPS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(32);
    pub const DEFAULT_EVERYONE_GROUP_MEMORY_ID: MemoryId = MemoryId::new(33);

    thread_local! {
       // Convert HashMap to StableBTreeMap for groups by ID
        pub(crate) static GROUPS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<GroupID, Group, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(GROUPS_BY_ID_MEMORY_ID))
            )
        );
        
        // Convert Vec to StableVec for groups by time
        pub(crate) static GROUPS_BY_TIME_LIST: RefCell<StableVec<GroupID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(GROUPS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize GROUPS_BY_TIME_LIST")
        );
        
        // Convert RefCell<GroupID> to StableCell for default group
        pub(crate) static DEFAULT_EVERYONE_GROUP: RefCell<StableCell<GroupID, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DEFAULT_EVERYONE_GROUP_MEMORY_ID)),
                GroupID(generate_uuidv4(IDPrefix::Group))
            ).expect("Failed to initialize DEFAULT_EVERYONE_GROUP")
        );
    }

    // only call this after all other states have been initialized
    pub fn init_default_group() {
        let admin_invite_id = GroupInviteID(generate_uuidv4(IDPrefix::GroupInvite));
        let group_id = DEFAULT_EVERYONE_GROUP.with(|group_id| group_id.borrow().get().clone());
        let owner_id = OWNER_ID.with(|owner_id| owner_id.borrow().get().clone());
        let current_time = ic_cdk::api::time() / 1_000_000;
        let default_group = Group {
            id: group_id.clone(),
            name: "All Contacts".to_string(),
            owner: owner_id.clone(),
            avatar: None,
            private_note: None,
            public_note: Some("Default group for everyone in the drive".to_string()),
            admin_invites: vec![admin_invite_id.clone()], 
            member_invites: vec![admin_invite_id.clone()],
            created_at: current_time.clone(),
            last_modified_at: current_time.clone(),
            drive_id: DRIVE_ID.with(|drive_id| drive_id.clone()),
            endpoint_url: URL_ENDPOINT.with(|url| url.borrow().get().clone()),
            labels: Vec::new(),
            external_id: None,
            external_payload: None,
        };

        GROUPS_BY_ID_HASHTABLE.with(|groups| {
            groups.borrow_mut().insert(default_group.id.clone(), default_group.clone());
        });

        GROUPS_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(&default_group.id);
        });

        let admin_invite = GroupInvite {
            id: admin_invite_id.clone(),
            group_id: group_id.clone(),
            inviter_id: owner_id.clone(),
            invitee_id: GroupInviteeID::User(owner_id.clone()),
            role: GroupRole::Admin,
            note: "Default admin access".to_string(),
            active_from: current_time,
            expires_at: -1, // Never expires
            created_at: current_time,
            last_modified_at: current_time,
            redeem_code: None,
            from_placeholder_invitee: None,
            labels: Vec::new(),
            external_id: None,
            external_payload: None,
        };

        INVITES_BY_ID_HASHTABLE.with(|invites| {
            invites.borrow_mut().insert(admin_invite_id.clone(), admin_invite.clone());
        });

        let invitee_id = GroupInviteeID::User(owner_id.clone());
        USERS_INVITES_LIST_HASHTABLE.with(|users_invites| {
            let mut users_invites_ref = users_invites.borrow_mut();
            
            // Get the current GroupInviteIDList if it exists, or create a new one
            let mut invite_list = users_invites_ref.get(&invitee_id)
                .map_or_else(|| GroupInviteIDList::new(), |list| list.clone());
            
            // Add the new invitation to the list
            invite_list.add(admin_invite_id);  // Using add method instead of push
            
            // Store the updated list back in the hashtable
            users_invites_ref.insert(invitee_id, invite_list);
        });

        let tables = [
            SystemTableEnum::Drives,
            SystemTableEnum::Disks,
            SystemTableEnum::Contacts,
            SystemTableEnum::Groups,
            SystemTableEnum::Webhooks,
            SystemTableEnum::Labels,
            SystemTableEnum::Inbox
        ];
        
        for table in tables.iter() {
            let perm_id = SystemPermissionID(generate_uuidv4(IDPrefix::SystemPermission));
            let resource_id = SystemResourceID::Table(table.clone());
            let grantee_id = PermissionGranteeID::Group(group_id.clone());
            
            let permission = SystemPermission {
                id: perm_id.clone(),
                resource_id: resource_id.clone(),
                granted_to: grantee_id.clone(),
                granted_by: owner_id.clone(),
                permission_types: vec![SystemPermissionType::View],
                begin_date_ms: 0, // Active immediately
                expiry_date_ms: -1, // Never expires
                note: format!("Default VIEW permission for {} table", table),
                created_at: current_time,
                last_modified_at: current_time,
                redeem_code: None,
                from_placeholder_grantee: None,
                labels: Vec::new(),
                metadata: None,
                external_id: None,
                external_payload: None,
            };
            
            // Store in main hashtable
            SYSTEM_PERMISSIONS_BY_ID_HASHTABLE.with(|perms| {
                perms.borrow_mut().insert(perm_id.clone(), permission.clone());
            });
            
            // Add to resource index
            add_system_permission_to_resource(&resource_id, &perm_id);
            
            // Add to grantee index
            add_system_permission_to_grantee(&grantee_id, &perm_id);
            
            // Add to time-ordered list
            update_system_permissions_time_list(&perm_id, true);
        }
    }

    pub fn is_group_admin(user_id: &UserID, group_id: &GroupID) -> bool {
        GROUPS_BY_ID_HASHTABLE.with(|groups| {
            if let Some(group) = groups.borrow().get(group_id) {
                // Check if user is the owner
                if group.owner == *user_id {
                    return true;
                }

                // Check admin invites
                for invite_id in &group.admin_invites {
                    if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).clone()) {
                        if invite.invitee_id == GroupInviteeID::User(user_id.clone()) {
                            let current_time = ic_cdk::api::time();
                            if invite.active_from <= current_time && 
                               (invite.expires_at <= 0 || invite.expires_at > current_time as i64) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        })
    }

    pub fn is_user_on_local_group(user_id: &UserID, group: &Group) -> bool {
        // Check if user is the owner
        if group.owner == *user_id {
            return true;
        }
    
        // Check member invites (which includes admin invites)
        for invite_id in &group.member_invites {
            if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).clone()) {
                if invite.invitee_id == GroupInviteeID::User(user_id.clone()) {
                    let current_time = ic_cdk::api::time();
                    if invite.active_from <= current_time && 
                       (invite.expires_at <= 0 || invite.expires_at > current_time as i64) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub async fn is_user_on_group(user_id: &UserID, group_id: &GroupID) -> bool {
        let group_opt: Option<Group> = GROUPS_BY_ID_HASHTABLE.with(|groups| groups.borrow().get(group_id).clone());
        
        if let Some(group) = group_opt {
            // If it's our own drive's group, use local validation
            if group.endpoint_url == URL_ENDPOINT.with(|url| url.borrow().get().clone()) {
                return is_user_on_local_group(user_id, &group);
            }
    
            // It's an external group, make HTTP call to their validate endpoint
            let validation_url = format!("{}/groups/validate", group.endpoint_url.0.trim_end_matches('/'));
            
            let validation_body = json!({
                "group_id": group_id.0,
                "user_id": user_id.0,
            });
    
            let request = CanisterHttpRequestArgument {
                url: validation_url,
                method: HttpMethod::POST,
                headers: vec![
                    HttpHeader {
                        name: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    },
                ],
                body: Some(serde_json::to_vec(&validation_body).unwrap_or_default()),
                max_response_bytes: Some(2048),
                transform: None,
            };
    
            match http_request(request, 100_000_000_000).await {
                Ok((response,)) => {
                    if response.status.0 != BigUint::from_u16(200).unwrap_or_default() {
                        debug_log!("External group validation failed with status: {}", response.status.0);
                        return false;
                    }
    
                    match serde_json::from_slice::<ValidateGroupResponseData>(&response.body) {
                        Ok(result) => result.is_member,
                        Err(e) => {
                            debug_log!("Failed to parse group validation response: {}", e);
                            false
                        }
                    }
                },
                Err((code, msg)) => {
                    debug_log!("External group validation request failed: {:?} - {}", code, msg);
                    false
                }
            }
        } else {
            false
        }
    }
}


