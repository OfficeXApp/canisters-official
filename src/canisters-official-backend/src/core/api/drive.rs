// src/core/api/drive.rs
pub mod drive {
    use crate::{
        core::{
            api::{
                internals::drive_internals::{ensure_folder_structure, ensure_root_folder, format_file_asset_path, resolve_naming_conflict, sanitize_file_path, split_path, update_folder_file_uuids, update_subfolder_paths},
                types::DirectoryError,
                uuid::generate_unique_id
            },
            state::{
                directory::{
                    state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid},
                    types::{DriveFullFilePath, FileMetadata, FileUUID, FolderMetadata, FolderUUID}
                },
                disks::types::{DiskID, DiskTypeEnum},
            }, types::{ICPPrincipalString, PublicKeyBLS, UserID},
        }, rest::{directory::types::{DirectoryListResponse, FileConflictResolutionEnum, ListDirectoryRequest}, webhooks::types::SortDirection}
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
        disk_id: DiskID,
        user_id: UserID,
        file_size: u64,
        expires_at: i64,
        canister_id: String,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
    ) -> Result<FileMetadata, String> {
        let sanitized_file_path = sanitize_file_path(&file_path);
        let (folder_path, file_name) = split_path(&sanitized_file_path);
        
        // Handle naming conflicts
        let (final_name, final_path) = resolve_naming_conflict(
            &folder_path,
            &file_name,
            false,
            file_conflict_resolution.clone()
        );

        // If empty strings returned, it means we should keep the original file
        if final_name.is_empty() && final_path.is_empty() {
            if let Some(existing_uuid) = full_file_path_to_uuid.get(&DriveFullFilePath(sanitized_file_path.clone())) {
                return get_file_by_id(existing_uuid.clone());
            }
        }
        
        let full_file_path = final_path;
        let new_file_uuid = FileUUID(generate_unique_id("FileID", ""));

        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };

        let folder_uuid = ensure_folder_structure(&folder_path, disk_id.clone(), user_id.clone(), canister_icp_principal_string.clone());

        let existing_file_uuid = full_file_path_to_uuid.get(&DriveFullFilePath(full_file_path.clone())).map(|uuid| uuid.clone());

        // Handle version-related logic
        let (file_version, prior_version) = if let Some(existing_uuid) = &existing_file_uuid {
            match file_conflict_resolution {
                Some(FileConflictResolutionEnum::REPLACE) => {
                    let existing_file = file_uuid_to_metadata.get(existing_uuid).unwrap();
                    (existing_file.file_version + 1, Some(existing_uuid.clone()))
                },
                Some(FileConflictResolutionEnum::KEEP_NEWER) => {
                    let existing_file = file_uuid_to_metadata.get(existing_uuid).unwrap();
                    if existing_file.last_updated_date_ms > ic_cdk::api::time() / 1_000_000 {
                        return Ok(existing_file.clone());
                    }
                    (existing_file.file_version + 1, Some(existing_uuid.clone()))
                },
                _ => (1, None) // For KEEP_BOTH and KEEP_ORIGINAL, we create a new version chain
            }
        } else {
            (1, None)
        };

        let extension = file_name.rsplit('.').next().unwrap_or("").to_string();

        let file_metadata = FileMetadata {
            id: new_file_uuid.clone(),
            name: file_name,
            folder_uuid: folder_uuid.clone(),
            file_version,
            prior_version,
            next_version: None,
            extension: extension.clone(),
            full_file_path: DriveFullFilePath(full_file_path.clone()),
            tags: Vec::new(),
            created_by: user_id.clone(),
            created_date_ms: ic_cdk::api::time() / 1_000_000,
            disk_id,
            file_size,
            raw_url: format_file_asset_path(new_file_uuid.clone(), extension),
            last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
            last_updated_by: user_id,
            deleted: false,
            canister_id: ICPPrincipalString(PublicKeyBLS(canister_icp_principal_string.clone())),
            expires_at,
        };

        // Update version chain if we're replacing
        if let Some(existing_uuid) = existing_file_uuid {
            match file_conflict_resolution {
                Some(FileConflictResolutionEnum::REPLACE) | Some(FileConflictResolutionEnum::KEEP_NEWER) => {
                    // Update the prior version's next_version pointer
                    file_uuid_to_metadata.with_mut(|map| {
                        if let Some(existing_file) = map.get_mut(&existing_uuid) {
                            existing_file.next_version = Some(new_file_uuid.clone());
                        }
                    });
                    
                    // Remove old file from parent folder's file_uuids
                    update_folder_file_uuids(&folder_uuid, &existing_uuid, false);
                },
                _ => ()
            }
        }

        // Update hashtables
        file_uuid_to_metadata.insert(new_file_uuid.clone(), file_metadata.clone());
        full_file_path_to_uuid.insert(DriveFullFilePath(full_file_path), new_file_uuid.clone());

        // Update parent folder's file_uuids
        update_folder_file_uuids(&folder_uuid, &new_file_uuid, true);

        Ok(file_metadata)
    }

    pub fn create_folder(
        full_folder_path: DriveFullFilePath,
        disk_id: DiskID,
        user_id: UserID,
        expires_at: i64,
        canister_id: String,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
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
        if storage_part != disk_id.to_string() {
            return Err(String::from("Storage location mismatch"));
        }
    
        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };
    
        // Check if folder already exists
        if let Some(existing_folder_uuid) = full_folder_path_to_uuid.get(&DriveFullFilePath(sanitized_path.clone())) {
            match file_conflict_resolution.unwrap_or(FileConflictResolutionEnum::KEEP_BOTH) {
                FileConflictResolutionEnum::REPLACE => {
                    // Delete existing folder and create new one
                    let mut deleted_files = Vec::new();
                    let mut deleted_folders = Vec::new();
                    delete_folder(&existing_folder_uuid, &mut deleted_folders, &mut deleted_files)?;
                },
                FileConflictResolutionEnum::KEEP_NEWER => {
                    // Compare timestamps and keep newer one
                    if let Some(existing_folder) = folder_uuid_to_metadata.get(&existing_folder_uuid) {
                        if existing_folder.last_updated_date_ms > ic_cdk::api::time() / 1_000_000 {
                            return Ok(existing_folder);
                        }
                        // Delete older folder
                        let mut deleted_files = Vec::new();
                        let mut deleted_folders = Vec::new();
                        delete_folder(&existing_folder_uuid, &mut deleted_folders, &mut deleted_files)?;
                    }
                },
                FileConflictResolutionEnum::KEEP_ORIGINAL => {
                    // Return existing folder
                    return folder_uuid_to_metadata
                        .get(&existing_folder_uuid)
                        .map(|metadata| metadata.clone())
                        .ok_or_else(|| "Existing folder not found".to_string());
                },
                FileConflictResolutionEnum::KEEP_BOTH => {
                    // Split the path into parent path and folder name
                    let path_parts: Vec<&str> = folder_path.split('/').filter(|&x| !x.is_empty()).collect();
                    let parent_path = if path_parts.len() > 1 {
                        format!("{}::{}/", storage_part, path_parts[..path_parts.len()-1].join("/"))
                    } else {
                        format!("{}::", storage_part)
                    };
                    let folder_name = path_parts.last().unwrap_or(&"");
    
                    // Generate new name with suffix
                    let (_, new_path) = resolve_naming_conflict(
                        &parent_path,
                        folder_name,
                        true,
                        Some(FileConflictResolutionEnum::KEEP_BOTH),
                    );
                    sanitized_path = new_path;
                }
            }
        }
    
        // Create the folder and get its UUID
        let new_folder_uuid = ensure_folder_structure(
            &sanitized_path,
            disk_id,
            user_id.clone(),
            canister_icp_principal_string,
        );
    
        // Update the metadata with the correct expires_at value
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(&new_folder_uuid) {
                folder.expires_at = expires_at;
            }
        });
    
        // Get and return the updated folder metadata
        folder_uuid_to_metadata
            .get(&new_folder_uuid)
            .map(|metadata| metadata.clone())
            .ok_or_else(|| "Failed to get created folder metadata".to_string())
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
                folder.name = new_name;
                folder.full_folder_path = DriveFullFilePath(new_folder_path.clone());
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
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
                file.name = new_name.clone();
                file.full_file_path = DriveFullFilePath(new_path.clone());
                file.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
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

    pub fn delete_folder(
        folder_id: &FolderUUID,
        all_deleted_folders: &mut Vec<FolderUUID>,
        all_deleted_files: &mut Vec<FileUUID>,
    ) -> Result<(), String> {
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

        // Add this folder's files to the deleted files list, respecting the limit
        for file_id in file_ids.clone() {
            if all_deleted_files.len() >= 2000 {
                break;
            }
            ic_cdk::println!("Deleting file: {}", file_id);
            if let Ok(()) = delete_file(&file_id) {
                all_deleted_files.push(file_id);
            }
        }

        // First delete files in the current folder
        ic_cdk::println!("Deleting files in the folder");
        for file_id in file_ids {
            ic_cdk::println!("Deleting file: {}", file_id);
            if let Err(e) = delete_file(&file_id) {
                ic_cdk::println!("Error deleting file {}: {}", file_id, e);
                return Err(format!("Failed to delete file {}: {}", file_id, e));
            }
            
            // Only add to deleted files list after successful deletion
            if all_deleted_files.len() < 2000 {
                all_deleted_files.push(file_id);
            }
        }
    
        // Recursively delete subfolders, passing the same vectors
        for subfolder_id in subfolder_ids {
            if let Err(e) = delete_folder(&subfolder_id, all_deleted_folders, all_deleted_files) {
                ic_cdk::println!("Error deleting subfolder {}: {}", subfolder_id, e);
            }
        }
    
    
        // Mark the folder as deleted
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(folder_id) {
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                folder.deleted = true;
            }
        });
        
        // Remove folder path mapping
        ic_cdk::println!("Removing folder path from full_folder_path_to_uuid");
        full_folder_path_to_uuid.remove(&folder_path);

        // Add to deleted folders list after successful deletion
        if all_deleted_folders.len() < 2000 {
            all_deleted_folders.push(folder_id.clone());
        }
    
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