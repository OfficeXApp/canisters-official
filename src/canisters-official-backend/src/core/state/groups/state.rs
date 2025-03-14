// src/core/state/groups/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;
    use crate::{debug_log, rest::groups::types::{ValidateGroupResponseData}};
    use serde_json::json;

    use crate::core::{state::{drives::state::state::URL_ENDPOINT, group_invites::{state::state::INVITES_BY_ID_HASHTABLE, types::{GroupInviteID, GroupInviteeID}}, groups::types::{Group, GroupID}}, types::UserID};
    
    thread_local! {
        // default is to use the api key id to lookup the api key
        pub(crate) static GROUPS_BY_ID_HASHTABLE: RefCell<HashMap<GroupID, Group>> = RefCell::new(HashMap::new());
        // track in hashtable users list of ApiKeyIDs
        pub(crate) static GROUPS_BY_TIME_LIST: RefCell<Vec<GroupID>> = RefCell::new(Vec::new());
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


