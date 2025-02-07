pub mod drive {
    use crate::{
        core::{
            api::{
                internals::drive_internals::{ensure_folder_structure, ensure_root_folder, sanitize_file_path, split_path, update_folder_file_uuids, update_subfolder_paths},
                types::DirectoryError,
                uuid::generate_unique_id
            },
            state::{
                directory::{
                    state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid},
                    types::{DriveFullFilePath, FileMetadata, FileUUID, FolderMetadata, FolderUUID}
                },
                disks::types::DiskTypeEnum,
            }, types::{ICPPrincipalString, PublicKeyBLS, UserID},
        }, rest::{directory::types::{DirectoryListResponse, ListDirectoryRequest}, webhooks::types::SortDirection}
    };

    pub fn fetch_files_at_folder_path(config: ListDirectoryRequest) -> Result<DirectoryListResponse, DirectoryError> {
        let ListDirectoryRequest { 
            folder_id, 
            path, 
            filters: _, 
            page_size, 
            direction, 
            cursor 
        } = config;
    
        // Get the folder UUID either from folder_id or path
        let folder_uuid = if let Some(id) = folder_id {
            FolderUUID(id)
        } else if let Some(path_str) = path {
            full_folder_path_to_uuid
                .get(&DriveFullFilePath(path_str.clone()))
                .ok_or_else(|| DirectoryError::FolderNotFound(format!("Path not found: {}", path_str)))?
        } else {
            return Err(DirectoryError::FolderNotFound("Neither folder_id nor path provided".to_string()));
        };
    
        // Get folder metadata
        let folder = folder_uuid_to_metadata
            .get(&folder_uuid)
            .ok_or_else(|| DirectoryError::FolderNotFound("Folder metadata not found".to_string()))?;
    
        let total_folders = folder.subfolder_uuids.len();
        let total_files = folder.file_uuids.len();
        let total_items = total_folders + total_files;
    
        // Parse cursor to get starting position
        let start_pos = cursor
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);
    
        // Determine range based on direction and cursor
        let range_start = match direction {
            SortDirection::Asc => start_pos,
            SortDirection::Desc => start_pos.saturating_sub(page_size)
        };
    
        let mut folders = Vec::new();
        let mut files = Vec::new();
        let mut count = 0;
        let mut current_pos = range_start;
        
        // Fill results while tracking count
        while count < page_size && current_pos < total_items {
            if current_pos < total_folders {
                // Add folder
                if let Some(subfolder) = folder_uuid_to_metadata.get(&folder.subfolder_uuids[current_pos]) {
                    folders.push(subfolder);
                    count += 1;
                }
            } else {
                // Add file
                let file_index = current_pos - total_folders;
                if let Some(file) = file_uuid_to_metadata.get(&folder.file_uuids[file_index]) {
                    files.push(file);
                    count += 1;
                }
            }
            current_pos += 1;
        }
    
        // Generate next cursor if there are more items
        let next_cursor = if current_pos < total_items {
            Some(current_pos.to_string())
        } else {
            None
        };
    
        Ok(DirectoryListResponse {
            folders,
            files,
            total_folders,
            total_files,
            cursor: next_cursor,
        })
    }

    pub fn create_file(
        file_path: String,
        storage_location: DiskTypeEnum,
        user_id: UserID,
        expires_at: i64,
        canister_id: String,
    ) -> FileUUID {
        let sanitized_file_path = sanitize_file_path(&file_path);
        let full_file_path = sanitized_file_path;
        let new_file_uuid = FileUUID(generate_unique_id("FileID", ""));

        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };

        let (folder_path, file_name) = split_path(&full_file_path);
        let folder_uuid = ensure_folder_structure(&folder_path, storage_location.clone(), user_id.clone(), canister_icp_principal_string.clone());

        let existing_file_uuid = full_file_path_to_uuid.get(&DriveFullFilePath(full_file_path.clone())).map(|uuid| uuid.clone());

        let file_version = if let Some(existing_uuid) = &existing_file_uuid {
            let existing_file = file_uuid_to_metadata.get(existing_uuid).unwrap();
            existing_file.file_version + 1
        } else {
            1
        };

        let extension = file_name.rsplit('.').next().unwrap_or("").to_string();

        let file_metadata = FileMetadata {
            id: new_file_uuid.clone(),
            original_file_name: file_name,
            folder_uuid: folder_uuid.clone(),
            file_version,
            prior_version: existing_file_uuid.clone(),
            next_version: None,
            extension,
            full_file_path: DriveFullFilePath(full_file_path.clone()),
            tags: Vec::new(),
            owner: user_id,
            created_date: ic_cdk::api::time(),
            storage_location,
            file_size: 0,
            raw_url: String::new(),
            last_changed_unix_ms: ic_cdk::api::time() / 1_000_000,
            deleted: false,
            canister_id: ICPPrincipalString(PublicKeyBLS(canister_icp_principal_string.clone())),
            expires_at,
        };

        // Update hashtables
        file_uuid_to_metadata.insert(new_file_uuid.clone(), file_metadata);
        full_file_path_to_uuid.insert(DriveFullFilePath(full_file_path), new_file_uuid.clone());

        // Update parent folder's file_uuids
        update_folder_file_uuids(&folder_uuid, &new_file_uuid, true);

        // Update prior version if it exists
        if let Some(existing_uuid) = existing_file_uuid {
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(existing_file) = map.get_mut(&existing_uuid) {
                    existing_file.next_version = Some(new_file_uuid.clone());
                }
            });
            // Remove the old file UUID from the parent folder
            update_folder_file_uuids(&folder_uuid, &existing_uuid, false);
        }

        new_file_uuid
    }

    pub fn create_folder(
        full_folder_path: DriveFullFilePath,
        storage_location: DiskTypeEnum,
        user_id: UserID,
        expires_at: i64,
        canister_id: String,
    ) -> Result<FolderMetadata, String> {
        // Ensure the path ends with a slash
        let mut sanitized_path = sanitize_file_path(&full_folder_path.to_string());
        if !sanitized_path.ends_with('/') {
            sanitized_path.push('/');
        }
    
        if sanitized_path.is_empty() {
            return Err(String::from("Invalid folder path"));
        }
    
        // Split the path into storage and folder parts
        let parts: Vec<&str> = sanitized_path.split("::").collect();
        if parts.len() < 2 {
            return Err(String::from("Invalid folder path format"));
        }
    
        let storage_part = parts[0];
        let folder_path = parts[1..].join("::");
    
        // Ensure the storage location matches
        if storage_part != storage_location.to_string() {
            return Err(String::from("Storage location mismatch"));
        }
    
        // Split the folder path into individual parts
        let path_parts: Vec<&str> = folder_path.split('/').filter(|&x| !x.is_empty()).collect();

        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };
    
        let mut current_path = format!("{}::", storage_part);
        let mut parent_folder_uuid = ensure_root_folder(&storage_location, &user_id, canister_icp_principal_string.clone());

        // root folder case
        if path_parts.is_empty() {
            return folder_uuid_to_metadata
                .get(&parent_folder_uuid)
                .map(|metadata| metadata.clone())
                .ok_or_else(|| "Parent folder not found".to_string());
        }
    
        // Iterate through path parts and create folders as needed
        for (i, part) in path_parts.iter().enumerate() {
            current_path.push_str(part);
            current_path.push('/');
    
            if !full_folder_path_to_uuid.contains_key(&DriveFullFilePath(current_path.clone())) {
                let new_folder_uuid = FolderUUID(generate_unique_id("FolderID", ""));
                let new_folder = FolderMetadata {
                    id: new_folder_uuid.clone(),
                    original_folder_name: part.to_string(),
                    parent_folder_uuid: Some(parent_folder_uuid.clone()),
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
                    expires_at,
                };
    
                full_folder_path_to_uuid.insert(DriveFullFilePath(current_path.clone()), new_folder_uuid.clone());
                folder_uuid_to_metadata.insert(new_folder_uuid.clone(), new_folder.clone());
    
                // Update parent folder
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(parent_folder) = map.get_mut(&parent_folder_uuid) {
                        parent_folder.subfolder_uuids.push(new_folder_uuid.clone());
                    }
                });
    
                parent_folder_uuid = new_folder_uuid;
    
                // If this is the last part, return the created folder
                if i == path_parts.len() - 1 {
                    return Ok(new_folder);
                }
            } else {
                parent_folder_uuid = full_folder_path_to_uuid
                    .get(&DriveFullFilePath(current_path.clone()))
                    .expect("Failed to get parent folder UUID from path");
            }
        }
    
        // If we've reached here, it means the folder already existed
        Err(String::from("Folder already exists"))
    }

    pub fn get_file_by_id(file_id: FileUUID) -> Result<FileMetadata, String> {
        file_uuid_to_metadata
            .get(&file_id)
            .map(|metadata| metadata.clone())
            .ok_or_else(|| "File not found".to_string())
    }

    pub fn get_folder_by_id(folder_id: FolderUUID) -> Result<FolderMetadata, String> {
        folder_uuid_to_metadata
            .get(&folder_id)
            .map(|metadata| metadata.clone())
            .ok_or_else(|| "Folder not found".to_string())
    }

    pub fn rename_folder(folder_id: FolderUUID, new_name: String) -> Result<FolderUUID, String> {
        // Get current folder metadata
        let folder = folder_uuid_to_metadata
            .get(&folder_id)
            .ok_or_else(|| "Folder not found".to_string())?;
        
        let old_path = folder.full_folder_path.clone();
        ic_cdk::println!("Old folder path: {}", old_path);
    
        // Create owned String before splitting
        let path_string = old_path.to_string();
        
        // Split the path into storage and folder parts
        let parts: Vec<&str> = path_string.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Err("Invalid folder structure".to_string());
        }
    
        let storage_part = parts[0].to_string();
        let folder_path = parts[1].trim_end_matches('/').to_string();
    
        // Perform path manipulation
        let path_parts: Vec<&str> = folder_path.rsplitn(2, '/').collect();
        let (parent_path, _current_folder_name) = match path_parts.len() {
            2 => (path_parts[1].to_string(), path_parts[0].to_string()),
            1 => (String::new(), path_parts[0].to_string()),
            _ => return Err("Invalid folder structure".to_string()),
        };
    
        // Construct the new folder path
        let new_folder_path = if parent_path.is_empty() {
            format!("{}::{}{}", storage_part, new_name, "/")
        } else {
            format!("{}::{}/{}{}", storage_part, parent_path, new_name, "/")
        };
    
        // Check if a folder with the new path already exists
        if full_folder_path_to_uuid.contains_key(&DriveFullFilePath(new_folder_path.clone())) {
            return Err("A folder with the new name already exists in the parent directory".to_string());
        }
    
        // Update folder metadata using with_mut
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(&folder_id) {
                folder.original_folder_name = new_name;
                folder.full_folder_path = DriveFullFilePath(new_folder_path.clone());
                folder.last_changed_unix_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        // Update path mappings
        ic_cdk::println!("Removing old path from full_folder_path_to_uuid: {}", old_path);
        full_folder_path_to_uuid.remove(&old_path);
    
        ic_cdk::println!("Inserting new path into full_folder_path_to_uuid: {}", new_folder_path);
        full_folder_path_to_uuid.insert(DriveFullFilePath(new_folder_path.clone()), folder_id.clone());
    
        // Update subfolder paths recursively
        update_subfolder_paths(&folder_id, &old_path.to_string(), &new_folder_path);
    
        // Update parent folder reference if needed
        if !parent_path.is_empty() {
            let parent_full_path = format!("{}::{}", storage_part, parent_path);
            if let Some(parent_uuid) = full_folder_path_to_uuid.get(&DriveFullFilePath(parent_full_path.clone())) {
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(parent_folder) = map.get_mut(&parent_uuid) {
                        if !parent_folder.subfolder_uuids.contains(&folder_id) {
                            parent_folder.subfolder_uuids.push(folder_id.clone());
                            ic_cdk::println!("Added folder UUID to parent folder's subfolder_uuids");
                        }
                    }
                });
            } else {
                ic_cdk::println!("Parent folder not found for path: {}", parent_full_path);
                return Err("Parent folder not found".to_string());
            }
        }
    
        ic_cdk::println!("Folder renamed successfully");
        Ok(folder_id)
    }
    
    pub fn rename_file(file_id: FileUUID, new_name: String) -> Result<FileUUID, String> {
        ic_cdk::println!(
            "Attempting to rename file. File ID: {}, New Name: {}",
            file_id,
            new_name
        );
    
        // Get current file metadata
        let file = file_uuid_to_metadata
            .get(&file_id)
            .ok_or_else(|| "File not found".to_string())?;
        
        let old_path = file.full_file_path.clone();
        ic_cdk::println!("Old file path: {}", old_path);
    
        // Create owned String before splitting
        let path_string = old_path.to_string();
        
        // Split the path into storage part and the rest
        let parts: Vec<&str> = path_string.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Err("Invalid file structure".to_string());
        }
    
        let storage_part = parts[0].to_string();
        let file_path = parts[1].to_string();
    
        // Split the file path and replace the last part (file name)
        let path_parts: Vec<&str> = file_path.rsplitn(2, '/').collect();
        let new_path = if path_parts.len() > 1 {
            format!("{}::{}/{}", storage_part, path_parts[1], new_name)
        } else {
            format!("{}::{}", storage_part, new_name)
        };
    
        ic_cdk::println!("New file path: {}", new_path);
    
        // Check if a file with the new name already exists
        if full_file_path_to_uuid.contains_key(&DriveFullFilePath(new_path.clone())) {
            ic_cdk::println!("Error: A file with this name already exists");
            return Err("A file with this name already exists".to_string());
        }
    
        // Update file metadata
        file_uuid_to_metadata.with_mut(|map| {
            if let Some(file) = map.get_mut(&file_id) {
                file.original_file_name = new_name.clone();
                file.full_file_path = DriveFullFilePath(new_path.clone());
                file.last_changed_unix_ms = ic_cdk::api::time() / 1_000_000;
                file.extension = new_name
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .to_string();
                ic_cdk::println!("Updated file metadata: {:?}", file);
            }
        });
    
        // Update path mappings
        ic_cdk::println!(
            "Removing old path from full_file_path_to_uuid: {}",
            old_path
        );
        full_file_path_to_uuid.remove(&old_path);
    
        ic_cdk::println!(
            "Inserting new path into full_file_path_to_uuid: {}",
            new_path
        );
        full_file_path_to_uuid.insert(DriveFullFilePath(new_path), file_id.clone());
    
        ic_cdk::println!("File renamed successfully");
        Ok(file_id)
    }

    pub fn delete_folder(folder_id: &FolderUUID) -> Result<(), String> {
        ic_cdk::println!("Attempting to delete folder. Folder ID: {}", folder_id);
        
        // Get folder data before modifications
        let folder = folder_uuid_to_metadata
            .get(folder_id)
            .ok_or_else(|| {
                ic_cdk::println!("Error: Folder not found. Folder ID: {}", folder_id);
                "Folder not found".to_string()
            })?;
    
        let folder_path = folder.full_folder_path.clone();
        let subfolder_ids = folder.subfolder_uuids.clone();
        let file_ids = folder.file_uuids.clone();
        
        ic_cdk::println!("Folder found. Full path: {}", folder_path);
        
        // Remove folder path mapping
        ic_cdk::println!("Removing folder path from full_folder_path_to_uuid");
        full_folder_path_to_uuid.remove(&folder_path);
    
        // Recursively delete subfolders
        ic_cdk::println!("Deleting subfolders");
        for subfolder_id in subfolder_ids {
            ic_cdk::println!("Deleting subfolder: {}", subfolder_id);
            delete_folder(&subfolder_id)?;
        }
    
        // Delete files in this folder
        ic_cdk::println!("Deleting files in the folder");
        for file_id in file_ids {
            ic_cdk::println!("Deleting file: {}", file_id);
            delete_file(&file_id)?;
        }
    
        // Mark the folder as deleted
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(folder_id) {
                folder.last_changed_unix_ms = ic_cdk::api::time() / 1_000_000;
                folder.deleted = true;
            }
        });
    
        ic_cdk::println!("Folder deleted successfully");
        Ok(())
    }
    
    pub fn delete_file(file_id: &FileUUID) -> Result<(), String> {
        ic_cdk::println!("Attempting to delete file. File ID: {}", file_id);
        
        // Get file data before modifications
        let file = file_uuid_to_metadata
            .get(file_id)
            .ok_or_else(|| {
                ic_cdk::println!("Error: File not found. File ID: {}", file_id);
                "File not found".to_string()
            })?;
    
        let file_path = file.full_file_path.clone();
        // Use ref in pattern matching to avoid moving the values
        let ref prior_version = file.prior_version;
        let ref next_version = file.next_version;
        
        ic_cdk::println!("File found. Full path: {}", file_path);
        
        // Remove file path mapping
        ic_cdk::println!("Removing file path from full_file_path_to_uuid");
        full_file_path_to_uuid.remove(&file_path);
    
        // Handle versioning
        if let Some(ref prior_id) = prior_version {
            ic_cdk::println!("Updating prior version. Prior version ID: {}", prior_id);
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(prior_file) = map.get_mut(&prior_id) {
                    prior_file.next_version = next_version.clone();
                    ic_cdk::println!("Updated prior file's next_version: {:?}", prior_file.next_version);
                }
            });
        }
    
        if let Some(ref next_id) = next_version {
            ic_cdk::println!("Updating next version. Next version ID: {}", next_id);
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(next_file) = map.get_mut(next_id) {
                    next_file.prior_version = prior_version.clone();
                    ic_cdk::println!("Updated next file's prior_version: {:?}", next_file.prior_version);
                }
            });
        }
    
        // Remove file metadata
        file_uuid_to_metadata.remove(file_id);
    
        ic_cdk::println!("File deleted successfully");
        Ok(())
    }


}