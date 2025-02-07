
pub mod drive_internals {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FileUUID, FolderMetadata, FolderUUID}}, disks::types::DiskTypeEnum}, types::{ICPPrincipalString, PublicKeyBLS, UserID}}, debug_log, 
        
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

    pub fn ensure_root_folder(storage_location: &DiskTypeEnum, user_id: &UserID, canister_id: String,) -> FolderUUID {
        let root_path = DriveFullFilePath(format!("{}::", storage_location.to_string()));
        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };
        if let Some(uuid) = full_folder_path_to_uuid.get(&root_path) {
            uuid.clone()
        } else {
            let root_folder_uuid = generate_unique_id("FolderID", "");
            let root_folder = FolderMetadata {
                id: FolderUUID(root_folder_uuid.clone()),
                original_folder_name: String::new(),
                parent_folder_uuid: None,
                subfolder_uuids: Vec::new(),
                file_uuids: Vec::new(),
                full_folder_path: root_path.clone(),
                tags: Vec::new(),
                owner: user_id.clone(),
                created_date: ic_cdk::api::time(),
                storage_location: storage_location.clone(),
                last_changed_unix_ms: ic_cdk::api::time() / 1_000_000,
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
        storage_location: DiskTypeEnum,
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

        let mut parent_uuid = ensure_root_folder(&storage_location, &user_id, canister_icp_principal_string.clone());

        for part in path_parts[1].split('/').filter(|&p| !p.is_empty()) {
            current_path = format!("{}{}/", current_path.clone(), part);
            
            if !full_folder_path_to_uuid.contains_key(&DriveFullFilePath(current_path.clone())) {
                let new_folder_uuid = FolderUUID(generate_unique_id("FolderID",""));
                let new_folder = FolderMetadata {
                    id: new_folder_uuid.clone(),
                    original_folder_name: part.to_string(),
                    parent_folder_uuid: Some(parent_uuid.clone()),
                    subfolder_uuids: Vec::new(),
                    file_uuids: Vec::new(),
                    full_folder_path: DriveFullFilePath(current_path.clone()),
                    tags: Vec::new(),
                    owner: user_id.clone(),
                    created_date: ic_cdk::api::time(),
                    storage_location: storage_location.clone(),
                    last_changed_unix_ms: ic_cdk::api::time() / 1_000_000,
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
}