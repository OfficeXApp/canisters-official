// src/core/api/internals.rs
pub mod drive_internals {
    use std::collections::{HashSet, VecDeque};

    use crate::{
        core::{api::{drive::drive::get_folder_by_id, helpers::get_appropriate_url_endpoint, permissions::directory::derive_breadcrumb_visibility_previews, types::{DirectoryError, DirectoryIDError}, uuid::{generate_uuidv4, mark_claimed_uuid}}, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FileID, FolderID, FolderRecord, PathTranslationResponse}}, disks::{state::state::DISKS_BY_ID_HASHTABLE, types::{AwsBucketAuth, DiskID, DiskTypeEnum}}, drives::{state::state::DRIVE_ID, types::{DriveID, ExternalID, ExternalPayload}}, group_invites::{state::state::{INVITES_BY_ID_HASHTABLE, USERS_INVITES_LIST_HASHTABLE}, types::GroupInviteeID}, groups::{state::state::GROUPS_BY_ID_HASHTABLE, types::GroupID}, permissions::{state::state::{DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE, DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE, DIRECTORY_PERMISSIONS_BY_RESOURCE_HASHTABLE}, types::{DirectoryPermission, DirectoryPermissionType, PermissionGranteeID, PlaceholderPermissionGranteeID, PUBLIC_GRANTEE_ID}}}, types::{ClientSuggestedUUID, ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::directory::types::{DirectoryListResponse, DirectoryResourceID, FileConflictResolutionEnum, FilePathBreadcrumb, ListDirectoryRequest}, 
        
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
        // Don't trim the leading slash, only trailing
        let sanitized = sanitized.trim_end_matches('/').to_string();
    
        let final_path = format!("{}::{}", storage_part, sanitized);
        ic_cdk::println!("sanitize_file_path: {} -> {}", original, final_path);
        final_path
    }
    

    pub fn ensure_root_folder(disk_id: &DiskID, disk_type: &DiskTypeEnum,user_id: &UserID, drive_id: DriveID) -> FolderID {
        let root_path = DriveFullFilePath(format!("{}::/", disk_id.to_string()));
        let root_uuid = if let Some(uuid) = full_folder_path_to_uuid.get(&root_path) {
            uuid.clone()
        } else {
            let root_folder_uuid = generate_uuidv4(IDPrefix::Folder);
            let root_folder = FolderRecord {
                id: FolderID(root_folder_uuid.clone()),
                name: String::new(),
                parent_folder_uuid: None,
                restore_trash_prior_folder_uuid: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_directory_path: root_path.clone(),
                labels: Vec::new(),
                created_by: user_id.clone(),
                created_at: ic_cdk::api::time() / 1_000_000,
                disk_id: disk_id.clone(),
                disk_type: disk_type.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: user_id.clone(),
                deleted: false,
                drive_id: drive_id.clone(),
                expires_at: -1,
                has_sovereign_permissions: false,
                shortcut_to: None,
                external_id: None,
                external_payload: None,
                notes: None,
            };
    
            full_folder_path_to_uuid.insert(root_path, FolderID(root_folder_uuid.clone()));
            folder_uuid_to_metadata.insert(FolderID(root_folder_uuid.clone()), root_folder);
            FolderID(root_folder_uuid)
        };

        // Ensure .trash folder exists
        let trash_path = DriveFullFilePath(format!("{}::.trash/", disk_id.to_string()));
        if !full_folder_path_to_uuid.contains_key(&trash_path) {
            let trash_folder_uuid = generate_uuidv4(IDPrefix::Folder);
            let trash_folder = FolderRecord {
                id: FolderID(trash_folder_uuid.clone()),
                name: ".trash".to_string(),
                parent_folder_uuid: Some(root_uuid.clone()),
                restore_trash_prior_folder_uuid: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_directory_path: trash_path.clone(),
                labels: Vec::new(),
                created_by: user_id.clone(),
                created_at: ic_cdk::api::time() / 1_000_000,
                disk_id: disk_id.clone(),
                disk_type: disk_type.clone(),
                last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                last_updated_by: user_id.clone(),
                deleted: false,
                drive_id: drive_id.clone(),
                expires_at: -1,
                has_sovereign_permissions: true,
                shortcut_to: None,
                external_id: None,
                external_payload: None,
                notes: None,
            };

            full_folder_path_to_uuid.insert(trash_path, FolderID(trash_folder_uuid.clone()));
            folder_uuid_to_metadata.insert(FolderID(trash_folder_uuid.clone()), trash_folder);
            
            // Add trash folder to root's subfolders
folder_uuid_to_metadata.with_mut(|map| {
    if let Some(mut root_folder) = map.get(&root_uuid) {
        root_folder.subfolder_uuids.push(FolderID(trash_folder_uuid));
        map.insert(root_uuid.clone(), root_folder);
    }
});
        }

        root_uuid
    }

    pub fn update_subfolder_paths(folder_id: &FolderID, old_path: &str, new_path: &str) {
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
                subfolder.full_directory_path.clone()
            } else {
                continue;
            };
    
            let new_subfolder_path = DriveFullFilePath(old_subfolder_path.to_string().replace(old_path, new_path));
            
            // Update folder metadata
            folder_uuid_to_metadata.with_mut(|map| {
                if let Some(mut subfolder) = map.get(subfolder_id) {
                    subfolder.full_directory_path = new_subfolder_path.clone();
                    map.insert(subfolder_id.clone(), subfolder);
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
                file.full_directory_path.clone()
            } else {
                continue;
            };
    
            let new_file_path = DriveFullFilePath(old_file_path.to_string().replace(old_path, new_path));
            
            // Update file metadata
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(mut file) = map.get(file_id) {
                    file.full_directory_path = new_file_path.clone();
                    map.insert(file_id.clone(), file);
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
        disk_type: DiskTypeEnum,
        user_id: UserID,
        drive_id: DriveID,
        has_sovereign_permissions: bool,
        external_id: Option<ExternalID>,
        external_payload: Option<ExternalPayload>,
        final_folder_id: Option<ClientSuggestedUUID>,
        shortcut_to: Option<FolderID>,
        notes: Option<String>,
    ) -> FolderID {
        let path_parts: Vec<&str> = folder_path.split("::").collect();
        let mut current_path = format!("{}::/", path_parts[0]);  // Always start with root slash

        let mut parent_uuid = ensure_root_folder(&disk_id, &disk_type, &user_id, DRIVE_ID.with(|id| id.clone()));

        for part in path_parts[1].split('/').filter(|&p| !p.is_empty()) {
            current_path = format!("{}{}/", current_path.clone(), part);
            
            if !full_folder_path_to_uuid.contains_key(&DriveFullFilePath(current_path.clone())) {
                let is_final_folder= part == path_parts[1].split('/').filter(|&p| !p.is_empty()).last().unwrap();
               
                let new_folder_uuid = match is_final_folder {
                    true => {
                        match final_folder_id.clone() {
                            Some(id) => FolderID(id.to_string()),
                            None => FolderID(generate_uuidv4(IDPrefix::Folder)),
                        }
                    },
                    false => FolderID(generate_uuidv4(IDPrefix::Folder)),
                };

                let new_folder = FolderRecord {
                    id: new_folder_uuid.clone(),
                    name: part.to_string(),
                    parent_folder_uuid: Some(parent_uuid.clone()),
                    subfolder_uuids: Vec::new(),
                    file_uuids: Vec::new(),
                    full_directory_path: DriveFullFilePath(current_path.clone()),
                    labels: Vec::new(),
                    created_by: user_id.clone(),
                    created_at: ic_cdk::api::time() / 1_000_000,
                    disk_id: disk_id.clone(),
                    disk_type: disk_type.clone(),
                    last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
                    last_updated_by: user_id.clone(),
                    deleted: false,
                    drive_id: drive_id.clone(),
                    expires_at: -1,
                    restore_trash_prior_folder_uuid: None,
                    shortcut_to: if is_final_folder {
                        shortcut_to.clone()
                    } else {
                        None
                    },
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
                        external_payload.clone()
                    } else {
                        None
                    },
                    notes: if is_final_folder {
                        notes.clone()
                    } else {
                        None
                    },
                };

                full_folder_path_to_uuid.insert(DriveFullFilePath(current_path.clone()), new_folder_uuid.clone());
                folder_uuid_to_metadata.insert(new_folder_uuid.clone(), new_folder);

                mark_claimed_uuid(&new_folder_uuid.clone().to_string());

                // Update parent folder's subfolder_uuids
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(mut parent_folder) = map.get(&parent_uuid) {
                        if !parent_folder.subfolder_uuids.contains(&new_folder_uuid) {
                            parent_folder.subfolder_uuids.push(new_folder_uuid.clone());
                        }
                        map.insert(parent_uuid.clone(), parent_folder);
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
    

    pub fn update_folder_file_uuids(folder_uuid: &FolderID, file_uuid: &FileID, is_add: bool) {
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(mut folder) = map.get(folder_uuid) {
                if is_add {
                    if !folder.file_uuids.contains(file_uuid) {
                        folder.file_uuids.push(file_uuid.clone());
                    }
                } else {
                    folder.file_uuids.retain(|uuid| uuid != file_uuid);
                }
                map.insert(folder_uuid.clone(), folder);
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
        file_uuid: FileID,
        extension: String,
    ) -> String {
        let base_url = get_appropriate_url_endpoint();
        let drive_id = DRIVE_ID.with(|id| id.clone());
        format!(
            "{}/v1/{}/directory/asset/{file_uuid}.{extension}",
            base_url,
            drive_id,
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
                        format!("{}/{} ({})", base_path.trim_end_matches('/'), final_name, counter)
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
        folder_id: Option<FolderID>, 
        folder_path: Option<DriveFullFilePath>,
        disk_id: DiskID,
        disk_type: DiskTypeEnum,
        user_id: UserID,
        drive_id: DriveID,
    ) -> Result<FolderRecord, String> {
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
                    disk_type,
                    user_id,
                    drive_id,
                    false,
                    None,
                    None,
                    None,
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

    pub fn is_user_in_group(user_id: &UserID, group_id: &GroupID) -> bool {
        // First check if group exists and user is owner
        let is_owner = GROUPS_BY_ID_HASHTABLE.with(|groups| {
            groups.borrow()
                .get(group_id)
                .map(|group| group.owner == *user_id)
                .unwrap_or(false)
        });
    
        if is_owner {
            return true;
        }
    
        // Get all user's invites first (outside the other with block)
        let user_invites = USERS_INVITES_LIST_HASHTABLE.with(|user_invites| {
            user_invites.borrow()
                .get(&GroupInviteeID::User(user_id.clone()))
                .map(|list| list.clone())
                .unwrap_or_default()
        });
    
        // Now check if any of the user's invites are active for this group
        let now = ic_cdk::api::time() / 1_000_000;
        INVITES_BY_ID_HASHTABLE.with(|invites| {
            user_invites.iter().any(|invite_id| {
                if let Some(invite) = invites.borrow().get(invite_id) {
                    // Check if invite is for this group
                    invite.group_id == *group_id && 
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

    pub async fn fetch_root_shortcuts_of_user(
        config: &ListDirectoryRequest,
        user_id: &UserID,
    ) -> Result<DirectoryListResponse, DirectoryError> {

        debug_log!("Fetching root shortcuts of user: {:?}", user_id);

        // Ensure disk_id is provided
        let _disk_id = config
            .disk_id
            .as_ref()
            .ok_or(DirectoryError::FolderNotFound("DiskID not provided".to_string()))?;
        let disk_id = DiskID(_disk_id.clone());
    
        // Fetch permissions (user, group, public)
        let user_permissions = DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|permissions| {
            permissions
                .borrow()
                .get(&PermissionGranteeID::User(user_id.clone()))
                .clone()
                .unwrap_or_default()
        });
    
        let group_permissions = USERS_INVITES_LIST_HASHTABLE.with(|invites| {
            invites
                .borrow()
                .get(&GroupInviteeID::User(user_id.clone()))
                .map(|list| list.invites.clone())
                .unwrap_or_default()
                .into_iter()  // This is fine as it's working with a Vec<InviteID>
                .filter_map(|invite_id| {
                    INVITES_BY_ID_HASHTABLE.with(|invites| {
                        invites
                            .borrow()
                            .get(&invite_id)
                            .map(|invite| invite.group_id.clone())
                    })
                })
                .flat_map(|group_id| {
                    DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|permissions| {
                        permissions
                            .borrow()
                            .get(&PermissionGranteeID::Group(group_id))
                            .clone()  // Use cloned() instead of clone() here
                            .unwrap_or_default()
                            .permissions  // Access the permissions field directly
                            .into_iter()  // Now this works with Vec<DirectoryPermissionID>
                    })
                })
                .collect::<Vec<_>>()
        });
    
        let public_permissions = DIRECTORY_GRANTEE_PERMISSIONS_HASHTABLE.with(|permissions| {
            permissions
                .borrow()
                .get(&PermissionGranteeID::Public)
                .clone()
                .unwrap_or_default()
        });
    
        // Combine all permissions
        let mut all_permissions = user_permissions.permissions;
        all_permissions.extend(group_permissions);
        all_permissions.extend(public_permissions.permissions);
    
        // Fetch actual permission records and filter by disk_id
        let mut permission_records = Vec::new();
    
        for permission_id in all_permissions {
            if let Some(record) = DIRECTORY_PERMISSIONS_BY_ID_HASHTABLE.with(|permissions| {
                permissions.borrow().get(&permission_id).clone()
            }) {
                let resource_disk_matches = match &record.resource_id {
                    DirectoryResourceID::Folder(folder_id) => folder_uuid_to_metadata
                        .get(folder_id)
                        .map(|f| &f.disk_id == &disk_id)
                        .unwrap_or(false),
                    DirectoryResourceID::File(file_id) => file_uuid_to_metadata
                        .get(file_id)
                        .map(|f| &f.disk_id == &disk_id)
                        .unwrap_or(false),
                };
    
                if resource_disk_matches {
                    permission_records.push(record);
                }
            }
        }
    
        // Sort filtered permissions by last_modified_at descending
        permission_records.sort_by(|a, b| b.last_modified_at.cmp(&a.last_modified_at));
    
        // Paginate the filtered results
        let cursor_index = config.cursor.as_ref().and_then(|cursor| {
            permission_records
                .iter()
                .position(|record| record.id.to_string() == *cursor)
        });
    
        let start = cursor_index.map(|idx| idx + 1).unwrap_or(0);
        let end = (start + config.page_size).min(permission_records.len());
    
        let paginated_records = &permission_records[start..end];
    
        // Fetch corresponding folders and files after pagination
        let mut folders = Vec::new();
        let mut files = Vec::new();
    
        for record in paginated_records {
            match &record.resource_id {
                DirectoryResourceID::Folder(folder_id) => {
                    if let Some(folder) = folder_uuid_to_metadata.get(folder_id) {
                        folders.push(folder.clone());
                    }
                }
                DirectoryResourceID::File(file_id) => {
                    if let Some(file) = file_uuid_to_metadata.get(file_id) {
                        files.push(file.clone());
                    }
                }
            }
        }
    
        // Await async cast_fe conversions
        let mut folders_fe = Vec::new();
        for folder in folders {
            folders_fe.push(folder.cast_fe(user_id).await);
        }
    
        let mut files_fe = Vec::new();
        for file in files {
            files_fe.push(file.cast_fe(user_id).await);
        }

        // breadcrumbs of disk root folder and 'shared with me' folder
        let mut breadcrumbs = VecDeque::new();
        breadcrumbs.push_front(FilePathBreadcrumb {
            resource_id: "shared-with-me".to_string(),
            resource_name: "Shared with me".to_string(),
            visibility_preview: vec![]
        });
        let disk = DISKS_BY_ID_HASHTABLE.with(|map| map.borrow().get(&disk_id).map(|d| d.clone()));
        if let Some(disk) = disk {
            breadcrumbs.push_front(FilePathBreadcrumb {
                resource_id: disk.root_folder.clone().to_string(),
                resource_name: disk.name.clone(),
                visibility_preview: derive_breadcrumb_visibility_previews(DirectoryResourceID::Folder(disk.root_folder.clone()))
            });
        }
    
        // Construct response
        let response = DirectoryListResponse {
            folders: folders_fe.clone(),
            files: files_fe.clone(),
            total_files: files_fe.len(),
            total_folders: folders_fe.len(),
            breadcrumbs: breadcrumbs.into(),
            cursor: paginated_records
                .last()
                .map(|record| record.id.to_string()),
            permission_previews: [].to_vec(),
        };
    
        Ok(response)
    }
    
}