// src/core/api/internals.rs
pub mod drive_internals {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FileUUID, FolderMetadata, FolderUUID, PathTranslationResponse}}, disks::types::{DiskID, DiskTypeEnum}}, types::{ICPPrincipalString, PublicKeyBLS, UserID}}, debug_log, rest::directory::types::FileConflictResolutionEnum, 
        
    };
    
    use regex::Regex;

    pub fn sanitize_file_path(file_path: &str) -> String {
        let mut parts = file_path.splitn(2, "::");
        let storage_part = parts.next().unwrap_or("");
        let path_part = parts.next().unwrap_or("");
    
        let sanitized = path_part.replace(':', ";");

        // Compile a regex to match one or more consecutive slashes
        let re = Regex::new(r"/+").unwrap();
        let sanitized = re.replace_all(&sanitized, "/").to_string();

        // Remove leading and trailing slashes
        let sanitized = sanitized.trim_matches('/').to_string();

        // Additional sanitization can be performed here if necessary
    
        // Reconstruct the full path
        format!("{}::{}", storage_part, sanitized)
    }

    pub fn ensure_root_folder(disk_id: &DiskID, user_id: &UserID, canister_id: String,) -> FolderUUID {
        let root_path = DriveFullFilePath(format!("{}::", disk_id.to_string()));
        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };
        if let Some(uuid) = full_folder_path_to_uuid.get(&root_path) {
            uuid.clone()
        } else {
            let root_folder_uuid = generate_unique_id("FolderUUID", "");
            let root_folder = FolderMetadata {
                id: FolderUUID(root_folder_uuid.clone()),
                name: String::new(),
                parent_folder_uuid: None,
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
                canister_id: ICPPrincipalString(PublicKeyBLS(canister_icp_principal_string)),
                expires_at: -1,
            };

            full_folder_path_to_uuid.insert(root_path, FolderUUID(root_folder_uuid.clone()));
            folder_uuid_to_metadata.insert(FolderUUID(root_folder_uuid.clone()), root_folder);

            FolderUUID(root_folder_uuid)
        }
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
                let new_folder_uuid = FolderUUID(generate_unique_id("FolderUUID",""));
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
                    canister_id: ICPPrincipalString(PublicKeyBLS(canister_icp_principal_string.clone())),
                    expires_at: -1,
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
        let parts: Vec<&str> = full_path.rsplitn(2, '/').collect();
        match parts.as_slice() {
            [file_name, folder_path] => (folder_path.to_string(), file_name.to_string()),
            [single_part] => {
                let storage_parts: Vec<&str> = single_part.splitn(2, "::").collect();
                match storage_parts.as_slice() {
                    [storage, file_name] => (format!("{}::", storage), file_name.to_string()),
                    _ => (String::new(), single_part.to_string()),
                }
            },
            _ => (String::new(), String::new()),
        }
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
            "https://{}.raw.icp0.io/asset/{file_uuid}.{extension}",
            ic_cdk::api::id().to_text()
        )
    }

    pub fn resolve_naming_conflict(
        base_path: &str,
        name: &str,
        is_folder: bool,
        resolution: Option<FileConflictResolutionEnum>,
    ) -> (String, String) {
        let mut final_name = name.to_string();
        let mut final_path = if is_folder {
            format!("{}{}/", base_path, name)
        } else {
            format!("{}{}", base_path, name)
        };
    
        match resolution.unwrap_or(FileConflictResolutionEnum::KEEP_BOTH) {
            FileConflictResolutionEnum::REPLACE => (final_name, final_path),
            FileConflictResolutionEnum::KEEP_ORIGINAL => {
                if (is_folder && full_folder_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone()))) ||
                   (!is_folder && full_file_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone()))) {
                    return (String::new(), String::new()); // Signal to keep original
                }
                (final_name, final_path)
            }
            FileConflictResolutionEnum::KEEP_NEWER => {
                // For KEEP_NEWER, we'll implement timestamp comparison in the caller
                (final_name, final_path)
            }
            FileConflictResolutionEnum::KEEP_BOTH => {
                let mut counter = 1;
                while (is_folder && full_folder_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone()))) ||
                      (!is_folder && full_file_path_to_uuid.contains_key(&DriveFullFilePath(final_path.clone()))) {
                    counter += 1;
                    
                    // Split name and extension for files
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
                        format!("{}{}/", base_path, final_name)
                    } else {
                        format!("{}{}", base_path, final_name)
                    };
                }
                (final_name, final_path)
            }
        }
    }
}