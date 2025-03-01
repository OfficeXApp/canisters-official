// src/core/api/internals.rs
pub mod drive_internals {
    use std::collections::HashSet;

    use crate::{
        core::{api::{drive::drive::get_folder_by_id, types::DirectoryIDError, uuid::generate_unique_id}, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FileUUID, FolderMetadata, FolderUUID, PathTranslationResponse}}, disks::types::{AwsBucketAuth, DiskID, DiskTypeEnum}, drives::types::{ExternalID, ExternalPayload}, permissions::{state::state::{DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE}, types::{DirectoryPermission, DirectoryPermissionType, PermissionGranteeID, PlaceholderPermissionGranteeID, PUBLIC_GRANTEE_ID}}, team_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::TeamInviteeID}, teams::{state::state::TEAMS_BY_ID_HASHTABLE, types::TeamID}}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID, EXTERNAL_PAYLOAD_MAX_LEN}}, debug_log, rest::directory::types::{DirectoryResourceID, FileConflictResolutionEnum}, 
        
    };
    
    use regex::Regex;

    pub fn sanitize_file_path(file_path: &str) -> String {
        let original = file_path.to_string();
        let mut parts = file_path.splitn(2, "::");
        let storage_part = parts.next().unwrap_or("");
        let path_part = parts.next().unwrap_or("");
    
        let sanitized = path_part.replace(':', ";");
        let re = Regex::new(r"/+").unwrap();
        let sanitized = re.replace_all(&sanitized, "/").to_string();
        let sanitized = sanitized.trim_matches('/').to_string();
    
        let final_path = format!("{}::{}", storage_part, sanitized);
        ic_cdk::println!("sanitize_file_path: {} -> {}", original, final_path);
        final_path
    }
    

    pub fn ensure_root_folder(disk_id: &DiskID, user_id: &UserID, canister_id: String,) -> FolderUUID {
        let root_path = DriveFullFilePath(format!("{}::", disk_id.to_string()));
        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };
        let root_uuid = if let Some(uuid) = full_folder_path_to_uuid.get(&root_path) {
            uuid.clone()
        } else {
            let root_folder_uuid = generate_unique_id(IDPrefix::Folder, "");
            let root_folder = FolderMetadata {
                id: FolderUUID(root_folder_uuid.clone()),
                name: String::new(),
                parent_folder_uuid: None,
                restore_trash_prior_folder_path: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_folder_path: root_path.clone(),
                tags: Vec::new(),
                created_by: user_id.clone(),
                created_date_ms: ic_cdk::api::time(),
                disk_id: disk_id.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: user_id.clone(),
                deleted: false,
                canister_id: ICPPrincipalString(PublicKeyICP(canister_icp_principal_string.clone())),
                expires_at: -1,
                has_sovereign_permissions: false,
                external_id: None,
                external_payload: None,
            };
    
            full_folder_path_to_uuid.insert(root_path, FolderUUID(root_folder_uuid.clone()));
            folder_uuid_to_metadata.insert(FolderUUID(root_folder_uuid.clone()), root_folder);
            FolderUUID(root_folder_uuid)
        };

        // Ensure .trash folder exists
        let trash_path = DriveFullFilePath(format!("{}::.trash/", disk_id.to_string()));
        if !full_folder_path_to_uuid.contains_key(&trash_path) {
            let trash_folder_uuid = generate_unique_id(IDPrefix::Folder, "");
            let trash_folder = FolderMetadata {
                id: FolderUUID(trash_folder_uuid.clone()),
                name: ".trash".to_string(),
                parent_folder_uuid: Some(root_uuid.clone()),
                restore_trash_prior_folder_path: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_folder_path: trash_path.clone(),
                tags: Vec::new(),
                created_by: user_id.clone(),
                created_date_ms: ic_cdk::api::time(),
                disk_id: disk_id.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: user_id.clone(),
                deleted: false,
                canister_id: ICPPrincipalString(PublicKeyICP(canister_icp_principal_string)),
                expires_at: -1,
                has_sovereign_permissions: true,
                external_id: None,
                external_payload: None,
            };

            full_folder_path_to_uuid.insert(trash_path, FolderUUID(trash_folder_uuid.clone()));
            folder_uuid_to_metadata.insert(FolderUUID(trash_folder_uuid.clone()), trash_folder);
            
            // Add trash folder to root's subfolders
            folder_uuid_to_metadata.with_mut(|map| {
                if let Some(root_folder) = map.get_mut(&root_uuid) {
                    root_folder.subfolder_uuids.push(FolderUUID(trash_folder_uuid));
                }
            });
        }

        root_uuid
    }

    pub fn update_subfolder_paths(folder_id: &FolderUUID, old_path: &str, new_path: &str) {
        // Get folder metadata first
        let folder = match folder_uuid_to_metadata.get(folder_id) {
            Some(f) => f,
            None => return,
        };
    
        // Clone the vectors we need to iterate over to avoid borrowing issues
        let subfolder_uuids = folder.subfolder_uuids.clone();
        let file_uuids = folder.file_uuids.clone();
    
        // Update subfolders
        for subfolder_id in &subfolder_uuids {
            // Get old path before updating
            let old_subfolder_path = if let Some(subfolder) = folder_uuid_to_metadata.get(subfolder_id) {
                subfolder.full_folder_path.clone()
            } else {
                continue;
            };
    
            let new_subfolder_path = DriveFullFilePath(old_subfolder_path.to_string().replace(old_path, new_path));
            
            // Update folder metadata
            folder_uuid_to_metadata.with_mut(|map| {
                if let Some(subfolder) = map.get_mut(subfolder_id) {
                    subfolder.full_folder_path = new_subfolder_path.clone();
                }
            });
            
            // Update path mappings
            full_folder_path_to_uuid.remove(&old_subfolder_path);
            full_folder_path_to_uuid.insert(new_subfolder_path.clone(), subfolder_id.clone());
            
            // Recursively update paths for this subfolder
            update_subfolder_paths(subfolder_id, &old_subfolder_path.to_string(), &new_subfolder_path.to_string());
        }
    
        // Update file paths
        for file_id in &file_uuids {
            // Get old path before updating
            let old_file_path = if let Some(file) = file_uuid_to_metadata.get(file_id) {
                file.full_file_path.clone()
            } else {
                continue;
            };
    
            let new_file_path = DriveFullFilePath(old_file_path.to_string().replace(old_path, new_path));
            
            // Update file metadata
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(file) = map.get_mut(file_id) {
                    file.full_file_path = new_file_path.clone();
                }
            });
            
            // Update path mappings
            full_file_path_to_uuid.remove(&old_file_path);
            full_file_path_to_uuid.insert(new_file_path, file_id.clone());
        }
    }

    pub fn ensure_folder_structure(
        folder_path: &str,
        disk_id: DiskID,
        user_id: UserID,
        canister_id: String,
        has_sovereign_permissions: bool,
        external_id: Option<ExternalID>,
        external_payload: Option<ExternalPayload>,
    ) -> FolderUUID {
        let path_parts: Vec<&str> = folder_path.split("::").collect();
        let mut current_path = format!("{}::", path_parts[0]);

        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };

        let mut parent_uuid = ensure_root_folder(&disk_id, &user_id, canister_icp_principal_string.clone());

        for part in path_parts[1].split('/').filter(|&p| !p.is_empty()) {
            current_path = format!("{}{}/", current_path.clone(), part);
            
            if !full_folder_path_to_uuid.contains_key(&DriveFullFilePath(current_path.clone())) {
                let is_final_folder= part == path_parts[1].split('/').filter(|&p| !p.is_empty()).last().unwrap();
                let new_folder_uuid = FolderUUID(generate_unique_id(IDPrefix::Folder,""));
                let new_folder = FolderMetadata {
                    id: new_folder_uuid.clone(),
                    name: part.to_string(),
                    parent_folder_uuid: Some(parent_uuid.clone()),
                    subfolder_uuids: Vec::new(),
                    file_uuids: Vec::new(),
                    full_folder_path: DriveFullFilePath(current_path.clone()),
                    tags: Vec::new(),
                    created_by: user_id.clone(),
                    created_date_ms: ic_cdk::api::time(),
                    disk_id: disk_id.clone(),
                    last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                    last_updated_by: user_id.clone(),
                    deleted: false,
                    canister_id: ICPPrincipalString(PublicKeyICP(canister_icp_principal_string.clone())),
                    expires_at: -1,
                    restore_trash_prior_folder_path: None,
                    // only set if its the final folder and has sovereign permissions
                    has_sovereign_permissions: if is_final_folder {
                        has_sovereign_permissions
                    } else {
                        false
                    },
                    // only set if its the final folder
                    external_id: if is_final_folder {
                        external_id.clone()
                    } else {
                        None
                    },
                    // only set if its the final folder
                    external_payload: if is_final_folder {
                        let external_payload_safe = external_payload.clone().map(|ext_payload| {
                            if serde_json::to_string(&ext_payload).unwrap().len() > EXTERNAL_PAYLOAD_MAX_LEN {
                                None
                            } else {
                                Some(ext_payload)
                            }
                        }).flatten();
                        external_payload_safe
                    } else {
                        None
                    },
                };

                full_folder_path_to_uuid.insert(DriveFullFilePath(current_path.clone()), new_folder_uuid.clone());
                folder_uuid_to_metadata.insert(new_folder_uuid.clone(), new_folder);

                // Update parent folder's subfolder_uuids
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(parent_folder) = map.get_mut(&parent_uuid) {
                        if !parent_folder.subfolder_uuids.contains(&new_folder_uuid) {
                            parent_folder.subfolder_uuids.push(new_folder_uuid.clone());
                        }
                    }
                });

                parent_uuid = new_folder_uuid;
            } else {
                parent_uuid = full_folder_path_to_uuid.get(&DriveFullFilePath(current_path.clone()))
                    .expect("Folder UUID not found")
                    .clone();
            }
        }

            

        parent_uuid
    }

    pub fn split_path(full_path: &str) -> (String, String) {
        let original = full_path.to_string();
        let parts: Vec<&str> = full_path.rsplitn(2, '/').collect();
        let (folder, filename) = match parts.as_slice() {
            [file_name, folder_path] => (folder_path.to_string(), file_name.to_string()),
            [single_part] => {
                let storage_parts: Vec<&str> = single_part.splitn(2, "::").collect();
                match storage_parts.as_slice() {
                    [storage, file_name] => (format!("{}::", storage), file_name.to_string()),
                    _ => (String::new(), single_part.to_string()),
                }
            },
            _ => (String::new(), String::new()),
        };
        ic_cdk::println!("split_path: {} -> ({}, {})", original, folder, filename);
        (folder, filename)
    }
    

    pub fn update_folder_file_uuids(folder_uuid: &FolderUUID, file_uuid: &FileUUID, is_add: bool) {
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(folder_uuid) {
                if is_add {
                    if !folder.file_uuids.contains(file_uuid) {
                        folder.file_uuids.push(file_uuid.clone());
                    }
                } else {
                    folder.file_uuids.retain(|uuid| uuid != file_uuid);
                }
            }
        });
    }
    
    pub fn translate_path_to_id(path: DriveFullFilePath) -> PathTranslationResponse {
        // Check if path ends with '/' to determine if we're looking for a folder
        let is_folder_path = path.0.ends_with('/');
        
        let mut response = PathTranslationResponse {
            folder: None,
            file: None,
        };

        if is_folder_path {
            // Look up folder UUID first
            if let Some(folder_uuid) = full_folder_path_to_uuid.get(&path) {
                // Then get the folder metadata
                response.folder = folder_uuid_to_metadata.get(&folder_uuid);
            }
        } else {
            // Look up file UUID first
            if let Some(file_uuid) = full_file_path_to_uuid.get(&path) {
                // Then get the file metadata
                response.file = file_uuid_to_metadata.get(&file_uuid);
            }
        }

        response
    }

    pub fn format_file_asset_path (
        file_uuid: FileUUID,
        extension: String,
    ) -> String {
        format!(
            "https://{}.raw.icp0.io/directory/asset/{file_uuid}.{extension}",
            ic_cdk::api::id().to_text()
        )
    }

    pub fn resolve_naming_conflict(
        base_path: &str,
        name: &str,
        is_folder: bool,
        resolution: Option<FileConflictResolutionEnum>,
    ) -> (String, String) {
        // Start with the initial name and computed full path.
        let mut final_name = name.to_string();
        let mut final_path = if is_folder {
            format!("{}/{}/", base_path.trim_end_matches('/'), final_name)
        } else {
            format!("{}/{}", base_path.trim_end_matches('/'), final_name)
        };
    
        debug_log!(
            "resolve_naming_conflict: initial base_path: '{}', name: '{}', is_folder: {} -> final_name: '{}', final_path: '{}'",
            base_path, name, is_folder, final_name, final_path
        );
    
        match resolution.unwrap_or(FileConflictResolutionEnum::KEEP_BOTH) {
            FileConflictResolutionEnum::REPLACE => {
                debug_log!(
                    "resolve_naming_conflict: Using REPLACE. Returning final_name: '{}' and final_path: '{}'",
                    final_name,
                    final_path
                );
                (final_name, final_path)
            },
            FileConflictResolutionEnum::KEEP_ORIGINAL => {
                if (is_folder && full_folder_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone())))
                    || (!is_folder && full_file_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone())))
                {
                    debug_log!(
                        "resolve_naming_conflict (KEEP_ORIGINAL): Conflict found for final_path: '{}'. Returning empty strings to keep original.",
                        final_path
                    );
                    return (String::new(), String::new()); // Signal to keep original
                }
                debug_log!(
                    "resolve_naming_conflict (KEEP_ORIGINAL): No conflict for final_path: '{}'. Returning final_name: '{}' and final_path: '{}'",
                    final_path, final_name, final_path
                );
                (final_name, final_path)
            },
            FileConflictResolutionEnum::KEEP_NEWER => {
                debug_log!(
                    "resolve_naming_conflict: Using KEEP_NEWER. Returning final_name: '{}' and final_path: '{}'",
                    final_name,
                    final_path
                );
                (final_name, final_path)
            },
            FileConflictResolutionEnum::KEEP_BOTH => {
                let mut counter = 1;
                while (is_folder && full_folder_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone())))
                    || (!is_folder && full_file_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone())))
                {
                    debug_log!(
                        "resolve_naming_conflict (KEEP_BOTH): Conflict for final_path: '{}'. Counter: {}",
                        final_path,
                        counter
                    );
                    counter += 1;
    
                    // Split name and extension for files.
                    let (base_name, ext) = if !is_folder && name.contains('.') {
                        let parts: Vec<&str> = name.rsplitn(2, '.').collect();
                        (parts[1], parts[0])
                    } else {
                        (name, "")
                    };
    
                    final_name = if ext.is_empty() {
                        format!("{} ({})", base_name, counter)
                    } else {
                        format!("{} ({}).{}", base_name, counter, ext)
                    };
    
                    final_path = if is_folder {
                        format!("{}/{}/", base_path.trim_end_matches('/'), final_name)
                    } else {
                        format!("{}/{}", base_path.trim_end_matches('/'), final_name)
                    };
    
                    debug_log!(
                        "resolve_naming_conflict (KEEP_BOTH): New computed final_name: '{}', final_path: '{}'",
                        final_name,
                        final_path
                    );
                }
                debug_log!(
                    "resolve_naming_conflict (KEEP_BOTH): Final resolved final_name: '{}', final_path: '{}'",
                    final_name,
                    final_path
                );
                (final_name, final_path)
            }
        }
    }
    
    

    // Helper function to get destination folder from either ID or path
    pub fn get_destination_folder(
        folder_id: Option<FolderUUID>, 
        folder_path: Option<DriveFullFilePath>,
        disk_id: DiskID,
        user_id: UserID,
        canister_id: String,
    ) -> Result<FolderMetadata, String> {
        if let Some(id) = folder_id {
            folder_uuid_to_metadata
                .get(&id)
                .clone()
                .ok_or_else(|| "Destination folder not found".to_string())
        } else if let Some(path) = folder_path {
            let translation = translate_path_to_id(path.clone());
            if let Some(folder) = translation.folder {
                Ok(folder)
            } else {
                // Folder not found at the given path; create the folder structure.
                let new_folder_uuid = ensure_folder_structure(
                    &path.to_string(),
                    disk_id,
                    user_id,
                    canister_id,
                    false,
                    None,
                    None,
                );
                // Retrieve the folder metadata using the new UUID.
                get_folder_by_id(new_folder_uuid)
            }
        } else {
            Err("Neither destination folder ID nor path provided".to_string())
        }
    }
    

    /// Validates that if the disk type is AwsBucket or StorjWeb3,
    /// then auth_json is provided and can be deserialized into AwsBucketAuth.
    pub fn validate_auth_json(disk_type: &DiskTypeEnum, auth_json: &Option<String>) -> Result<(), String> {
        if *disk_type == DiskTypeEnum::AwsBucket || *disk_type == DiskTypeEnum::StorjWeb3 {
            match auth_json {
                Some(json_str) => {
                    // Try to parse the provided JSON string into AwsBucketAuth.
                    serde_json::from_str::<AwsBucketAuth>(json_str)
                        .map_err(|e| format!("Invalid auth_json for {}: {}", disk_type, e))?;
                    Ok(())
                },
                None => Err(format!("auth_json is required for disk type {}", disk_type)),
            }
        } else {
            Ok(())
        }
    }

    pub fn is_user_in_team(user_id: &UserID, team_id: &TeamID) -> bool {
        // First check if team exists and user is owner
        let is_owner = TEAMS_BY_ID_HASHTABLE.with(|teams| {
            teams.borrow()
                .get(team_id)
                .map(|team| team.owner == *user_id)
                .unwrap_or(false)
        });
    
        if is_owner {
            return true;
        }
    
        // Check active invites for the user
        INVITES_BY_ID_HASHTABLE.with(|invites| {
            // Get all user's invites
            let user_invites = USERS_INVITES_LIST_HASHTABLE.with(|user_invites| {
                user_invites.borrow()
                    .get(&TeamInviteeID::User(user_id.clone()))
                    .cloned()
                    .unwrap_or_default()
            });
    
            // Check if any of the user's invites are active for this team
            let now = ic_cdk::api::time();
            user_invites.iter().any(|invite_id| {
                if let Some(invite) = invites.borrow().get(invite_id) {
                    // Check if invite is for this team
                    invite.team_id == *team_id && 
                    // Check if invite is active (not expired and after active_from)
                    invite.expires_at > 0 &&
                    now >= invite.active_from &&
                    now < invite.expires_at as u64
                } else {
                    false
                }
            })
        })
    }

    
}