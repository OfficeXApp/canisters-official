// src/core/state/groups/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;
    use crate::{core::{api::uuid::generate_uuidv4, state::{drives::state::state::{CANISTER_ID, DRIVE_ID, OWNER_ID}, group_invites::{state::state::USERS_INVITES_LIST_HASHTABLE, types::{GroupInvite, GroupRole}}, permissions::{state::state::{SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE, SYSTEM_PERMISSIONS_BY_ID_HASHTABLE, SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE, SYSTEM_PERMISSIONS_BY_TIME_LIST}, types::{PermissionGranteeID, SystemPermission, SystemPermissionID, SystemPermissionType, SystemResourceID, SystemTableEnum}}}, types::IDPrefix}, debug_log, rest::groups::types::ValidateGroupResponseData};
    use serde_json::json;

    use crate::core::{state::{drives::state::state::URL_ENDPOINT, group_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::{GroupInviteID, GroupInviteeID}}, groups::types::{Group, GroupID}}, types::UserID};
    
    thread_local! {
        // default is to use the api key id to lookup the api key
        pub(crate) static GROUPS_BY_ID_HASHTABLE: RefCell<HashMap<GroupID, Group>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub(crate) static GROUPS_BY_TIME_LIST: RefCell<Vec<GroupID>> = RefCell::new(Vec::new());
        // default group id
        pub(crate) static DEFAULT_EVERYONE_GROUP: RefCell<GroupID> = RefCell::new(GroupID(generate_uuidv4(IDPrefix::Group)));
    }

    // only call this after all other states have been initialized
    pub fn init_default_group() {
        let admin_invite_id = GroupInviteID(generate_uuidv4(IDPrefix::GroupInvite));
        let group_id = DEFAULT_EVERYONE_GROUP.with(|group_id| group_id.borrow().clone());
        let owner_id = OWNER_ID.with(|owner_id| owner_id.borrow().clone());
        let current_time = ic_cdk::api::time() / 1_000_000;
        let default_group = Group {
            id: group_id.clone(),
            name: "Everyone".to_string(),
            owner: owner_id.clone(),
            avatar: None,
            private_note: None,
            public_note: Some("Default group for everyone in the drive".to_string()),
            admin_invites: vec![admin_invite_id.clone()], 
            member_invites: vec![admin_invite_id.clone()],
            created_at: current_time.clone(),
            last_modified_at: current_time.clone(),
            drive_id: DRIVE_ID.with(|drive_id| drive_id.clone()),
            endpoint_url: URL_ENDPOINT.with(|url| url.borrow().clone()),
            labels: Vec::new(),
            external_id: None,
            external_payload: None,
        };

        GROUPS_BY_ID_HASHTABLE.with(|groups| {
            groups.borrow_mut().insert(default_group.id.clone(), default_group.clone());
        });

        GROUPS_BY_TIME_LIST.with(|list| {
            list.borrow_mut().push(default_group.id.clone());
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
            let user_invites = users_invites_ref.entry(invitee_id).or_insert_with(Vec::new);
            user_invites.push(admin_invite_id);
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
            SYSTEM_PERMISSIONS_BY_RESOURCE_HASHTABLE.with(|resource_perms| {
                let mut resource_perms_ref = resource_perms.borrow_mut();
                let perms_list = resource_perms_ref.entry(resource_id).or_insert_with(Vec::new);
                perms_list.push(perm_id.clone());
            });
            
            // Add to grantee index
            SYSTEM_GRANTEE_PERMISSIONS_HASHTABLE.with(|grantee_perms| {
                let mut grantee_perms_ref = grantee_perms.borrow_mut();
                let perms_list = grantee_perms_ref.entry(grantee_id).or_insert_with(Vec::new);
                perms_list.push(perm_id.clone());
            });
            
            // Add to time-ordered list
            SYSTEM_PERMISSIONS_BY_TIME_LIST.with(|list| {
                list.borrow_mut().push(perm_id);
            });
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
                    if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).cloned()) {
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
            if let Some(invite) = INVITES_BY_ID_HASHTABLE.with(|invites| invites.borrow().get(invite_id).cloned()) {
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
        let group_opt = GROUPS_BY_ID_HASHTABLE.with(|groups| groups.borrow().get(group_id).cloned());
        
        if let Some(group) = group_opt {
            // If it's our own drive's group, use local validation
            if group.endpoint_url == URL_ENDPOINT.with(|url| url.borrow().clone()) {
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


