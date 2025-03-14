// src/core/api/drive.rs
pub mod drive {
    use crate::{
        core::{
            api::{
                disks::{aws_s3::{copy_s3_object, generate_s3_upload_url}, storj_web3::generate_storj_upload_url}, internals::drive_internals::{ensure_folder_structure, ensure_root_folder, format_file_asset_path, resolve_naming_conflict, sanitize_file_path, split_path, translate_path_to_id, update_folder_file_uuids, update_subfolder_paths}, permissions::directory::preview_directory_permissions, types::DirectoryError, uuid::{generate_uuidv4, mark_claimed_uuid}
            },
            state::{
                directory::{
                    state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid},
                    types::{DriveFullFilePath, FileID, FileRecord, FolderID, FolderRecord}
                },
                disks::{state::state::DISKS_BY_ID_HASHTABLE, types::{AwsBucketAuth, DiskID, DiskTypeEnum}}, drives::{state::state::update_external_id_mapping, types::{ExternalID, ExternalPayload}},
            }, types::{ClientSuggestedUUID, ICPPrincipalString, IDPrefix, PublicKeyICP, UserID},
        }, debug_log, rest::{directory::types::{DirectoryActionResult, DirectoryListResponse, DirectoryResourceID, DiskUploadResponse, FileConflictResolutionEnum, GetFileResponse, GetFolderResponse, ListDirectoryRequest, ListGetFileResponse, ListGetFolderResponse, RestoreTrashPayload, RestoreTrashResponse}, webhooks::types::SortDirection}
    };

    pub async fn fetch_files_at_folder_path(config: ListDirectoryRequest, user_id: UserID) -> Result<DirectoryListResponse, DirectoryError> {
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
            FolderID(id)
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

        // After getting folder contents but before returning
        let mut folder_responses = Vec::new();
        let mut file_responses = Vec::new();

        for folder in folders {
            let resource_id = DirectoryResourceID::Folder(folder.id.clone());
            let your_permissions = preview_directory_permissions(&resource_id, &user_id);
            let folder_response = ListGetFolderResponse {
                folder,
                permissions: your_permissions,
                requester_id: user_id.clone(),
            };
            folder_responses.push(folder_response);
        }

        for file in files {
            let resource_id = DirectoryResourceID::File(file.id.clone());
            let your_permissions = preview_directory_permissions(&resource_id, &user_id);
            let file_response = ListGetFileResponse {
                file,
                permissions: your_permissions,
                requester_id: user_id.clone(),
            };
            file_responses.push(file_response);
        }
    
        Ok(DirectoryListResponse {
            folders: folder_responses,
            files: file_responses,
            total_folders,
            total_files,
            cursor: next_cursor,
        })
    }

    pub fn create_file(
        id: Option<ClientSuggestedUUID>,
        file_path: String,
        disk_id: DiskID,
        user_id: UserID,
        file_size: u64,
        expires_at: i64,
        canister_id: String,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
        has_sovereign_permissions: Option<bool>,
        external_id: Option<ExternalID>,
        external_payload: Option<ExternalPayload>,
    ) -> Result<(FileRecord, DiskUploadResponse), String> {
        let sanitized_file_path: String = sanitize_file_path(&file_path);
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
                // For KEEP_ORIGINAL we just return Err since we don't want to generate an upload URL
                return Err("File already exists and resolution is KEEP_ORIGINAL".to_string());
            }
        }

        // check for disk and return error if not found
        let disk = DISKS_BY_ID_HASHTABLE.with(|map| {
            map.borrow()
                .get(&disk_id)
                .cloned()
        }).ok_or_else(|| "Disk not found".to_string())?;

        // Get the disk and if it's not found, return an error
        let disk = DISKS_BY_ID_HASHTABLE.with(|map| {
            map.borrow()
                .get(&disk_id)
                .cloned()
        }).ok_or_else(|| "Disk not found".to_string())?;
        
        let full_file_path = final_path;
        
        let new_file_uuid = match id {
            Some(id) => FileID(id.to_string()),
            None => FileID(generate_uuidv4(IDPrefix::File)),
        };

        ic_cdk::println!(
            "Checking full path: {} -> {}",
            sanitized_file_path,
            full_file_path_to_uuid.get(&DriveFullFilePath(sanitized_file_path.clone())).is_some()
        );
        
    
        let canister_icp_principal_string = if canister_id.is_empty() {
            ic_cdk::api::id().to_text()
        } else {
            canister_id.clone()
        };
    
        let folder_uuid = ensure_folder_structure(
            &folder_path, 
            disk_id.clone(), 
            disk.disk_type,
            user_id.clone(), 
            canister_icp_principal_string.clone(),
            false,
            None,
            None,
            None
        );
    
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
                        return match get_file_by_id(existing_uuid.clone()) {
                            Ok(existing_file) => {
                                // Get disk info for S3 upload URL generation
                                let disk = DISKS_BY_ID_HASHTABLE.with(|map| {
                                    map.borrow()
                                        .get(&disk_id)
                                        .cloned()
                                }).ok_or_else(|| "Disk not found".to_string())?;
    
                                let aws_auth: AwsBucketAuth = serde_json::from_str(&disk.auth_json.ok_or_else(|| "Missing AWS credentials".to_string())?)
                                    .map_err(|_| "Invalid AWS credentials format".to_string())?;
    
                                // Example using an "existing file" upload.
                                let upload_response = match existing_file.disk_type {
                                    DiskTypeEnum::AwsBucket => {
                                        generate_s3_upload_url(
                                            &existing_uuid.0,           // file_id
                                            &existing_file.extension,   // file_extension
                                            &aws_auth.clone(),                  // AWS credentials
                                            file_size,
                                            3600
                                        )?
                                    }
                                    DiskTypeEnum::StorjWeb3 => {
                                        generate_storj_upload_url(
                                            &existing_uuid.0,           // file_id
                                            &existing_file.extension,   // file_extension
                                            &aws_auth.clone(),                // Storj credentials (make sure to define this)
                                            file_size,
                                            3600
                                        )?
                                    }
                                    // Optionally handle other disk types
                                    _ => {
                                        panic!(
                                            "Unsupported disk type for generating an upload URL: {:?}",
                                            existing_file.disk_type
                                        );
                                    }
                                };

                                println!("Upload response: {:?}", upload_response);

    
                                Ok((existing_file, upload_response))
                            },
                            Err(e) => Err(e)
                        };
                    }
                    (existing_file.file_version + 1, Some(existing_uuid.clone()))
                },
                _ => (1, None) // For KEEP_BOTH and KEEP_ORIGINAL, we create a new version chain
            }
        } else {
            (1, None)
        };
    
        let extension = file_name.rsplit('.').next().unwrap_or("").to_string();

    
        let file_metadata = FileRecord {
            id: new_file_uuid.clone(),
            name: file_name,
            folder_uuid: folder_uuid.clone(),
            file_version,
            prior_version,
            next_version: None,
            extension: extension.clone(),
            full_file_path: DriveFullFilePath(full_file_path.clone()),
            labels: Vec::new(),
            created_by: user_id.clone(),
            created_at: ic_cdk::api::time() / 1_000_000,
            disk_id: disk_id.clone(),
            disk_type: disk.disk_type.clone(),
            file_size,
            raw_url: format_file_asset_path(new_file_uuid.clone(), extension),
            last_updated_date_ms: ic_cdk::api::time() / 1_000_000,
            last_updated_by: user_id,
            deleted: false,
            canister_id: ICPPrincipalString(PublicKeyICP(canister_icp_principal_string.clone())),
            expires_at,
            restore_trash_prior_folder_path: None,
            has_sovereign_permissions: has_sovereign_permissions.unwrap_or(false),
            external_id: external_id.clone(),
            external_payload: external_payload.clone(),
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
    
        mark_claimed_uuid(&new_file_uuid.clone().to_string());

        // Update parent folder's file_uuids
        update_folder_file_uuids(&folder_uuid, &new_file_uuid, true);
    
        // Get disk info for S3 upload URL generation
        let disk = DISKS_BY_ID_HASHTABLE.with(|map| {
            map.borrow()
                .get(&disk_id)
                .cloned()
        }).ok_or_else(|| "Disk not found".to_string())?;
    
        if disk.disk_type != DiskTypeEnum::AwsBucket && disk.disk_type != DiskTypeEnum::StorjWeb3 {
            return Err("Only S3 buckets are supported for file uploads".to_string());
        }
    
        let aws_auth: AwsBucketAuth = serde_json::from_str(&disk.auth_json.ok_or_else(|| "Missing AWS credentials".to_string())?)
            .map_err(|_| "Invalid AWS credentials format".to_string())?;
    
        let upload_response = match file_metadata.disk_type {
            DiskTypeEnum::AwsBucket => {
                generate_s3_upload_url(
                    &new_file_uuid.0,                // file_id
                    file_metadata.extension.as_str(),// file_extension
                    &aws_auth.clone(),                       // AWS credentials
                    file_size,
                    3600
                )?
            }
            DiskTypeEnum::StorjWeb3 => {
                generate_storj_upload_url(
                    &new_file_uuid.0,                // file_id
                    file_metadata.extension.as_str(),// file_extension
                    &aws_auth.clone(),                     // Storj credentials
                    file_size,
                    3600
                )?
            }
            _ => {
                panic!(
                    "Unsupported disk type for generating an upload URL: {:?}",
                    file_metadata.disk_type
                );
            }
        };
        
        println!("Upload response: {:?}", upload_response);
            
        update_external_id_mapping(
            None,
            external_id,
            Some(new_file_uuid.0.clone())
        );
    
        Ok((file_metadata, upload_response))
    }

    pub fn create_folder(
        id: Option<ClientSuggestedUUID>,
        full_folder_path: DriveFullFilePath,
        disk_id: DiskID,
        user_id: UserID,
        expires_at: i64,
        canister_id: String,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
        has_sovereign_permissions: Option<bool>,
        external_id: Option<ExternalID>,
        external_payload: Option<ExternalPayload>,
    ) -> Result<FolderRecord, String> {
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

        // Check if disk exists and return error if not found
        let disk = DISKS_BY_ID_HASHTABLE.with(|map| {
            map.borrow()
                .get(&disk_id)
                .cloned()
        }).ok_or_else(|| "Disk not found".to_string())?;
        
    
        // Check if folder already exists
        if let Some(existing_folder_uuid) = full_folder_path_to_uuid.get(&DriveFullFilePath(sanitized_path.clone())) {
            match file_conflict_resolution.unwrap_or(FileConflictResolutionEnum::KEEP_BOTH) {
                FileConflictResolutionEnum::REPLACE => {
                    // Delete existing folder and create new one
                    let mut deleted_files = Vec::new();
                    let mut deleted_folders = Vec::new();
                    let permanent_delete = true;
                    delete_folder(&existing_folder_uuid, &mut deleted_folders, &mut deleted_files, permanent_delete)?;
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
                        let permanent_delete = true;
                        delete_folder(&existing_folder_uuid, &mut deleted_folders, &mut deleted_files, permanent_delete)?;
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
            disk.disk_type,
            user_id.clone(),
            canister_icp_principal_string,
            has_sovereign_permissions.unwrap_or(false),
            external_id.clone(),
            external_payload,
            id
        );
        update_external_id_mapping(
            None,
            external_id,
            Some(new_folder_uuid.0.clone())
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

    pub fn get_file_by_id(file_id: FileID) -> Result<FileRecord, String> {
        file_uuid_to_metadata
            .get(&file_id)
            .map(|metadata| metadata.clone())
            .ok_or_else(|| "File not found".to_string())
    }

    pub fn get_folder_by_id(folder_id: FolderID) -> Result<FolderRecord, String> {
        folder_uuid_to_metadata
            .get(&folder_id)
            .map(|metadata| metadata.clone())
            .ok_or_else(|| "Folder not found".to_string())
    }

    pub fn rename_folder(folder_id: FolderID, new_name: String) -> Result<FolderID, String> {
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
            let parent_full_path = format!("{}::{}{}", storage_part, parent_path, "/");
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
    
    pub fn rename_file(file_id: FileID, new_name: String) -> Result<FileID, String> {
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
        folder_id: &FolderID,
        all_deleted_folders: &mut Vec<FolderID>,
        all_deleted_files: &mut Vec<FileID>,
        permanent: bool,
    ) -> Result<DriveFullFilePath, String> {
        // Get folder metadata
        let folder = folder_uuid_to_metadata
            .get(folder_id)
            .ok_or_else(|| "Folder not found".to_string())?;
    
        // Prevent deletion of root and .trash folders
        if folder.parent_folder_uuid.is_none() || folder.name == ".trash" {
            return Err("Cannot delete root or .trash folders".to_string());
        }
    
        // If folder is already in trash, only allow permanent deletion
        if let Some(_) = folder.restore_trash_prior_folder_path {
            if !permanent {
                return Err("Cannot move to trash: item is already in trash".to_string());
            }
        }
    
        if permanent {
            // Permanent deletion logic
            let folder_path = folder.full_folder_path.clone();
            let subfolder_ids = folder.subfolder_uuids.clone();
            let file_ids = folder.file_uuids.clone();
    
            // Delete files
            for file_id in file_ids {
                if let Ok(_) = delete_file(&file_id, true) {
                    if all_deleted_files.len() < 2000 {
                        all_deleted_files.push(file_id);
                    }
                }
            }
    
            // Recursively delete subfolders
            for subfolder_id in subfolder_ids {
                if let Ok(_) = delete_folder(&subfolder_id, all_deleted_folders, all_deleted_files, true) {
                    if all_deleted_folders.len() < 2000 {
                        all_deleted_folders.push(subfolder_id);
                    }
                }
            }
    
            // Remove folder metadata and path mapping
            folder_uuid_to_metadata.remove(&folder_id.clone());
            full_folder_path_to_uuid.remove(&folder_path);

            // remove external_ids 
            update_external_id_mapping(
                folder.external_id,
                None,
                Some(folder_id.0.clone())
            );
    
            // Remove from parent's subfolder list
            if let Some(parent_id) = folder.parent_folder_uuid {
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(parent) = map.get_mut(&parent_id) {
                        parent.subfolder_uuids.retain(|id| id != folder_id);
                    }
                });
            }

            
            Ok(DriveFullFilePath("".to_string()))
        } else {
            // Move to trash logic
            // Get .trash folder UUID
            let trash_path = DriveFullFilePath(format!("{}::.trash/", folder.disk_id.to_string()));
            let trash_uuid = full_folder_path_to_uuid
                .get(&trash_path)
                .ok_or_else(|| "Trash folder not found".to_string())?;
    
            // Store original folder path before moving
            let original_folder_path = folder.full_folder_path.clone();
    
            // First, set restore_trash_prior_folder_path for the main folder and all its contents
            let mut stack = vec![folder_id.clone()];
            
            while let Some(current_folder_id) = stack.pop() {
                // First handle the folder's metadata
                let mut file_ids = Vec::new();
                let mut current_folder_path = None;
                
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(current_folder) = map.get_mut(&current_folder_id) {
                        if current_folder_id == *folder_id {
                            // Main folder gets the original parent path
                            current_folder.restore_trash_prior_folder_path = Some(original_folder_path.clone());
                        } else {
                            // Subfolders keep their current path
                            current_folder.restore_trash_prior_folder_path = Some(current_folder.full_folder_path.clone());
                        }
    
                        // Add subfolders to stack
                        stack.extend(current_folder.subfolder_uuids.clone());
                        // Get the file IDs for processing after we release this borrow
                        file_ids = current_folder.file_uuids.clone();
                        current_folder_path = Some(current_folder.full_folder_path.clone());
                    }
                });
    
                // Now set restore info for all files using file_uuid_to_metadata
                if let Some(folder_path) = current_folder_path {
                    for file_id in file_ids {
                        file_uuid_to_metadata.with_mut(|file_map| {
                            if let Some(file) = file_map.get_mut(&file_id) {
                                file.restore_trash_prior_folder_path = Some(folder_path.clone());
                            }
                        });
                    }
                }
            }
    
            // Get trash folder metadata
            let trash_folder = folder_uuid_to_metadata
                .get(&trash_uuid)
                .ok_or_else(|| "Trash folder metadata not found".to_string())?;
    
            // Move folder to .trash
            let moved_folder = move_folder(
                folder_id,
                &trash_folder,
                Some(FileConflictResolutionEnum::KEEP_BOTH),
            )?;
    
            // Add moved items to tracking vectors
            if all_deleted_folders.len() < 2000 {
                all_deleted_folders.push(folder_id.clone());
            }
    
            // Track all files in the moved folder structure
            let mut stack = vec![folder_id.clone()];
            while let Some(current_folder_id) = stack.pop() {
                if let Some(current_folder) = folder_uuid_to_metadata.get(&current_folder_id) {
                    // Add all files in current folder
                    for file_id in &current_folder.file_uuids {
                        if all_deleted_files.len() < 2000 {
                            all_deleted_files.push(file_id.clone());
                        }
                    }
    
                    // Add subfolders to stack and tracking
                    for subfolder_id in &current_folder.subfolder_uuids {
                        if all_deleted_folders.len() < 2000 {
                            all_deleted_folders.push(subfolder_id.clone());
                        }
                        stack.push(subfolder_id.clone());
                    }
                }
            }
    
            // Return the new path in trash
            Ok(moved_folder.full_folder_path)
        }
    }

    pub fn delete_file(file_id: &FileID, permanent: bool) -> Result<DriveFullFilePath, String> {
        // Get file metadata
        let file = file_uuid_to_metadata
            .get(file_id)
            .ok_or_else(|| "File not found".to_string())?;
    
        // If file is already in trash, only allow permanent deletion
        if let Some(_) = file.restore_trash_prior_folder_path {
            if !permanent {
                return Err("Cannot move to trash: item is already in trash".to_string());
            }
        }
        
        if permanent {
            // Permanent deletion logic
            let file_path = file.full_file_path.clone();
            let folder_uuid = file.folder_uuid.clone();
            
            // Handle version chain
            if let Some(prior_id) = &file.prior_version {
                file_uuid_to_metadata.with_mut(|map| {
                    if let Some(prior_file) = map.get_mut(prior_id) {
                        prior_file.next_version = file.next_version.clone();
                    }
                });
            }
    
            if let Some(next_id) = &file.next_version {
                file_uuid_to_metadata.with_mut(|map| {
                    if let Some(next_file) = map.get_mut(next_id) {
                        next_file.prior_version = file.prior_version.clone();
                    }
                });
            }
    
            // Remove metadata and path mapping
            file_uuid_to_metadata.remove(file_id);
            full_file_path_to_uuid.remove(&file_path);
    
            // Remove from parent folder's file list
            folder_uuid_to_metadata.with_mut(|map| {
                if let Some(folder) = map.get_mut(&folder_uuid) {
                    folder.file_uuids.retain(|id| id != file_id);
                }
            });

            // remove external ids
            update_external_id_mapping(
                file.external_id,
                None,
                Some(file_id.0.clone())
            );
    
            Ok(DriveFullFilePath("".to_string()))
        } else {
            // Move to trash
            // Store original folder path before moving
            let original_folder_path = DriveFullFilePath(format!("{}/", file.full_file_path.0.rsplitn(2, '/').nth(1).unwrap_or("")));
            
            // Get .trash folder UUID
            let trash_path = DriveFullFilePath(format!("{}::.trash/", file.disk_id.to_string()));
            let trash_uuid = full_folder_path_to_uuid
                .get(&trash_path)
                .ok_or_else(|| "Trash folder not found".to_string())?;
    
            // Set restore_trash_prior_folder_path BEFORE moving the file
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(file) = map.get_mut(file_id) {
                    file.restore_trash_prior_folder_path = Some(original_folder_path);
                }
            });
    
            // Get trash folder metadata
            let trash_folder = folder_uuid_to_metadata
                .get(&trash_uuid)
                .ok_or_else(|| "Trash folder metadata not found".to_string())?;
    
            // Move file to .trash
            let moved_file = move_file(
                file_id,
                &trash_folder,
                Some(FileConflictResolutionEnum::KEEP_BOTH),
            )?;
    
            // Return the new path in trash
            Ok(moved_file.full_file_path)
        }
    }

    pub fn copy_file(
        file_id: &FileID,
        destination_folder: &FolderRecord,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
        new_copy_id: Option<ClientSuggestedUUID>,
    ) -> Result<FileRecord, String> {
        // Get source file metadata
        let source_file = file_uuid_to_metadata
            .get(file_id)
            .ok_or_else(|| "Source file not found".to_string())?;

        // Check if source and destination are on the same disk
        if source_file.disk_id != destination_folder.disk_id {
            return Err("Cannot copy files between different disks".to_string());
        }

        // Construct new file path in destination
        let new_path = format!("{}{}", destination_folder.full_folder_path.0, source_file.name);
        
        // Handle naming conflicts
        let (final_name, final_path) = resolve_naming_conflict(
            &destination_folder.full_folder_path.0,
            &source_file.name,
            false,
            file_conflict_resolution,
        );

        // If empty strings returned, it means we should keep the original file
        if final_name.is_empty() && final_path.is_empty() {
            if let Some(existing_uuid) = full_file_path_to_uuid.get(&DriveFullFilePath(new_path)) {
                return Ok(file_uuid_to_metadata.get(&existing_uuid.clone()).unwrap().clone());
            }
        }

        // Generate new UUID for the copy

        let new_file_uuid = match new_copy_id {
            Some(id) => FileID(id.to_string()),
            None => FileID(generate_uuidv4(IDPrefix::File)),
        };


        // If this is an S3 or Storj bucket, perform copy operation
        if source_file.disk_type == DiskTypeEnum::AwsBucket || 
            source_file.disk_type == DiskTypeEnum::StorjWeb3 {
            // Get disk auth info
            let disk = DISKS_BY_ID_HASHTABLE.with(|map| {
                map.borrow()
                    .get(&source_file.disk_id)
                    .cloned()
            }).ok_or_else(|| "Disk not found".to_string())?;

            let aws_auth: AwsBucketAuth = serde_json::from_str(&disk.auth_json
                .ok_or_else(|| "Missing AWS credentials".to_string())?
            ).map_err(|_| "Invalid AWS credentials format".to_string())?;

            // Prepare S3 copy operation parameters
            let source_key = format!("{}", source_file.raw_url);
            let destination_key = format_file_asset_path(new_file_uuid.clone(), source_file.extension.clone());

            // Fire and forget - initiate copy operation without waiting
            ic_cdk::spawn(async move {
                match copy_s3_object(&source_key, &destination_key, &aws_auth).await {
                    Ok(_) => ic_cdk::println!("S3 copy completed successfully"),
                    Err(e) => ic_cdk::println!("S3 copy failed: {}", e)
                }
            });
        }


        // Create new metadata for the copy
        let mut new_file_metadata = source_file.clone();
        new_file_metadata.id = new_file_uuid.clone();
        new_file_metadata.name = final_name;
        new_file_metadata.folder_uuid = destination_folder.id.clone();
        new_file_metadata.full_file_path = DriveFullFilePath(final_path.clone());
        new_file_metadata.file_version = 1;
        new_file_metadata.prior_version = None;
        new_file_metadata.next_version = None;
        new_file_metadata.created_at = ic_cdk::api::time() / 1_000_000;
        new_file_metadata.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
        new_file_metadata.raw_url = format_file_asset_path(new_file_uuid.clone(), new_file_metadata.extension.clone());

        // Update metadata maps
        file_uuid_to_metadata.insert(new_file_uuid.clone(), new_file_metadata.clone());
        full_file_path_to_uuid.insert(DriveFullFilePath(final_path), new_file_uuid.clone());

        // Update destination folder's file list
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(&destination_folder.id) {
                folder.file_uuids.push(new_file_uuid.clone());
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });

        Ok(new_file_metadata)
    }

    pub fn copy_folder(
        folder_id: &FolderID,
        destination_folder: &FolderRecord,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
        new_copy_id: Option<ClientSuggestedUUID>,
    ) -> Result<FolderRecord, String> {
        // Get source folder metadata
        let source_folder = folder_uuid_to_metadata
            .get(folder_id)
            .ok_or_else(|| "Source folder not found".to_string())?;

         // Check if source and destination are on the same disk
        if source_folder.disk_id != destination_folder.disk_id {
            return Err("Cannot copy folders between different disks".to_string());
        }
        
        // Handle naming conflicts
        let (final_name, final_path) = resolve_naming_conflict(
            &destination_folder.full_folder_path.0,
            &source_folder.name,
            true,
            file_conflict_resolution.clone(),
        );
    
        // Generate new UUID for the copy
        let new_folder_uuid = match new_copy_id {
            Some(id) => FolderID(id.to_string()),
            None => FolderID(generate_uuidv4(IDPrefix::Folder)),
        };
    
        // Create new metadata for the copy
        let mut new_folder_metadata = source_folder.clone();
        new_folder_metadata.id = new_folder_uuid.clone();
        new_folder_metadata.name = final_name;
        new_folder_metadata.parent_folder_uuid = Some(destination_folder.id.clone());
        new_folder_metadata.full_folder_path = DriveFullFilePath(final_path.clone());
        new_folder_metadata.subfolder_uuids = Vec::new(); // Will be populated while copying subfolders
        new_folder_metadata.file_uuids = Vec::new(); // Will be populated while copying files
        new_folder_metadata.created_at = ic_cdk::api::time() / 1_000_000;
        new_folder_metadata.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
    
        // Update metadata maps
        folder_uuid_to_metadata.insert(new_folder_uuid.clone(), new_folder_metadata.clone());
        full_folder_path_to_uuid.insert(DriveFullFilePath(final_path), new_folder_uuid.clone());
    
        // Update destination folder's subfolder list
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(&destination_folder.id) {
                folder.subfolder_uuids.push(new_folder_uuid.clone());
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        // Recursively copy all subfolders
        for subfolder_id in &source_folder.subfolder_uuids {
            if let Ok(copied_subfolder) = copy_folder(subfolder_id, &new_folder_metadata, file_conflict_resolution.clone(), None) {
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(folder) = map.get_mut(&new_folder_uuid) {
                        folder.subfolder_uuids.push(copied_subfolder.id.clone());
                    }
                });
            }
        }
    
        // Copy all files in the folder
        for file_id in &source_folder.file_uuids {
            if let Ok(copied_file) = copy_file(file_id, &new_folder_metadata, file_conflict_resolution.clone(), None) {
                folder_uuid_to_metadata.with_mut(|map| {
                    if let Some(folder) = map.get_mut(&new_folder_uuid) {
                        folder.file_uuids.push(copied_file.id.clone());
                    }
                });
            }
        }
    
        Ok(new_folder_metadata)
    }
    
    pub fn move_file(
        file_id: &FileID,
        destination_folder: &FolderRecord,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
    ) -> Result<FileRecord, String> {
        // Get source file metadata
        let source_file = file_uuid_to_metadata
            .get(file_id)
            .ok_or_else(|| "Source file not found".to_string())?;
    
        // Check if source and destination are on the same disk
        if source_file.disk_id != destination_folder.disk_id {
            return Err("Cannot move files between different disks".to_string());
        }

        // Get source folder to update its file_uuids
        let source_folder_id = source_file.folder_uuid.clone();
        
        // Handle naming conflicts
        let (final_name, final_path) = resolve_naming_conflict(
            &destination_folder.full_folder_path.0,
            &source_file.name,
            false,
            file_conflict_resolution,
        );
    
        // If empty strings returned, keep original file
        if final_name.is_empty() && final_path.is_empty() {
            return Ok(source_file.clone());
        }
    
        // Remove old path mapping
        full_file_path_to_uuid.remove(&source_file.full_file_path);
    
        // Update file metadata
        file_uuid_to_metadata.with_mut(|map| {
            if let Some(file) = map.get_mut(file_id) {
                file.name = final_name;
                file.folder_uuid = destination_folder.id.clone();
                file.full_file_path = DriveFullFilePath(final_path.clone());
                file.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        // Update path mapping
        full_file_path_to_uuid.insert(DriveFullFilePath(final_path), file_id.clone());
    
        // Remove file from source folder
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(&source_folder_id) {
                folder.file_uuids.retain(|id| id != file_id);
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        // Add file to destination folder
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(&destination_folder.id) {
                folder.file_uuids.push(file_id.clone());
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        Ok(file_uuid_to_metadata.get(file_id).unwrap().clone())
    }
    
    pub fn move_folder(
        folder_id: &FolderID,
        destination_folder: &FolderRecord,
        file_conflict_resolution: Option<FileConflictResolutionEnum>,
    ) -> Result<FolderRecord, String> {
        // Get source folder metadata
        let source_folder = folder_uuid_to_metadata
            .get(folder_id)
            .ok_or_else(|| "Source folder not found".to_string())?;
        
        // Check if source and destination are on the same disk
        if source_folder.disk_id != destination_folder.disk_id {
            return Err("Cannot move folders between different disks".to_string());
        }
    
        // Check for circular reference
        let mut current_folder = Some(destination_folder.id.clone());
        while let Some(folder_id) = current_folder {
            if folder_id == source_folder.id {
                return Err("Cannot move folder into itself or its subdirectories".to_string());
            }
            current_folder = folder_uuid_to_metadata
                .get(&folder_id)
                .and_then(|folder| folder.parent_folder_uuid.clone());
        }
    
        // Handle naming conflicts via resolve_naming_conflict.
        let (final_name, final_path) = resolve_naming_conflict(
            &destination_folder.full_folder_path.0,
            &source_folder.name,
            true,
            file_conflict_resolution,
        );
    
        // If empty strings returned, keep original folder
        if final_name.is_empty() && final_path.is_empty() {
            return Ok(source_folder.clone());
        }
    
        let old_path = source_folder.full_folder_path.clone();
        
        // Update folder metadata using with_mut.
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(folder) = map.get_mut(folder_id) {
                folder.name = final_name.clone();
                folder.parent_folder_uuid = Some(destination_folder.id.clone());
                folder.full_folder_path = DriveFullFilePath(final_path.clone());
                folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        // *** NEW: Update the global path mapping for the folder itself ***
        debug_log!("move_folder: Removing old full folder path mapping: {}", old_path);
        full_folder_path_to_uuid.remove(&old_path);
        debug_log!("move_folder: Inserting new full folder path mapping: {}", final_path);
        full_folder_path_to_uuid.insert(DriveFullFilePath(final_path.clone()), folder_id.clone());
    
        // Update path mappings for all subfolders and files.
        update_subfolder_paths(folder_id, &old_path.0, &final_path);
    
        // Remove folder from old parent's subfolder list.
        if let Some(old_parent_id) = &source_folder.parent_folder_uuid {
            folder_uuid_to_metadata.with_mut(|map| {
                if let Some(parent) = map.get_mut(old_parent_id) {
                    parent.subfolder_uuids.retain(|id| id != folder_id);
                    parent.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                }
            });
        }
    
        // Add folder to new parent's subfolder list.
        folder_uuid_to_metadata.with_mut(|map| {
            if let Some(new_parent) = map.get_mut(&destination_folder.id) {
                new_parent.subfolder_uuids.push(folder_id.clone());
                new_parent.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
            }
        });
    
        let updated_folder = folder_uuid_to_metadata
            .get(folder_id)
            .ok_or_else(|| "Failed to retrieve updated folder metadata".to_string())?;
        debug_log!("move_folder: Finished moving folder. New metadata: {:?}", updated_folder);
    
        Ok(updated_folder.clone())
    }
    

    // In drive.rs, update restore_from_trash function
    pub fn restore_from_trash(
        resource_id: &str,
        payload: &RestoreTrashPayload,
    ) -> Result<DirectoryActionResult, String> {
        // Check if resource exists as a folder
        let folder_id = FolderID(resource_id.to_string());
        if let Some(folder) = folder_uuid_to_metadata.get(&folder_id) {
            // Verify folder is actually in trash
            if folder.restore_trash_prior_folder_path.is_none() {
                return Err("Folder is not in trash".to_string());
            }

            // Determine target restore location
            let target_folder = if let Some(restore_path) = &payload.restore_to_folder_path {
                // First try to find existing folder at the path
                let translation = translate_path_to_id(restore_path.clone());
                if let Some(existing_folder) = translation.folder {
                    existing_folder
                } else {
                    // Create the folder structure if it doesn't exist
                    let new_folder_uuid = ensure_folder_structure(
                        &restore_path.to_string(),
                        folder.disk_id.clone(),
                        folder.disk_type.clone(),
                        folder.created_by.clone(),
                        folder.canister_id.0.0.clone(),
                        folder.has_sovereign_permissions.clone(),
                        None,
                        None,
                        None
                    );
                    
                    folder_uuid_to_metadata
                        .get(&new_folder_uuid)
                        .ok_or_else(|| "Failed to create restore folder path".to_string())?
                }
            } else {
                // Get the folder UUID from the stored path
                let path = folder.restore_trash_prior_folder_path.clone().unwrap();
                let translation = translate_path_to_id(path.clone());
                if let Some(existing_folder) = translation.folder {
                    existing_folder
                } else {
                    // Create the folder structure if original path doesn't exist
                    let new_folder_uuid = ensure_folder_structure(
                        &path.to_string(),
                        folder.disk_id.clone(),
                        folder.disk_type.clone(),
                        folder.created_by.clone(),
                        folder.canister_id.0.0.clone(),
                        folder.has_sovereign_permissions.clone(),
                        None,
                        None,
                        None
                    );
                    
                    folder_uuid_to_metadata
                        .get(&new_folder_uuid)
                        .ok_or_else(|| "Failed to create restore folder path".to_string())?
                }
            };

            // Verify target folder is not in trash
            if target_folder.restore_trash_prior_folder_path.is_some() {
                return Err(format!("Cannot restore to a folder that is in trash. Please first restore {}", target_folder.full_folder_path).to_string());
            }

            // Move folder to target location
            let restored_folder = move_folder(
                &folder_id,
                &target_folder,
                payload.file_conflict_resolution.clone(),
            )?;

            // Clear restore_trash_prior_folder_path for the folder and all its contents
            let mut stack = vec![folder_id.clone()];
            let mut restored_folders = vec![folder_id.clone()];
            let mut restored_files = Vec::new();

            while let Some(current_folder_id) = stack.pop() {
                if let Some(current_folder) = folder_uuid_to_metadata.get(&current_folder_id) {
                    // Process subfolders
                    for subfolder_id in &current_folder.subfolder_uuids {
                        folder_uuid_to_metadata.with_mut(|map| {
                            if let Some(subfolder) = map.get_mut(subfolder_id) {
                                subfolder.restore_trash_prior_folder_path = None;
                            }
                        });
                        restored_folders.push(subfolder_id.clone());
                        stack.push(subfolder_id.clone());
                    }

                    // Process files
                    for file_id in &current_folder.file_uuids {
                        file_uuid_to_metadata.with_mut(|map| {
                            if let Some(file) = map.get_mut(file_id) {
                                file.restore_trash_prior_folder_path = None;
                            }
                        });
                        restored_files.push(file_id.clone());
                    }
                }
            }

            // Clear restore_trash_prior_folder_path for the main folder
            folder_uuid_to_metadata.with_mut(|map| {
                if let Some(folder) = map.get_mut(&folder_id) {
                    folder.restore_trash_prior_folder_path = None;
                }
            });

            Ok(DirectoryActionResult::RestoreTrash(RestoreTrashResponse {
                restored_folders,
                restored_files,
            }))
        }
        // Handle file restore case similarly
        else if let Some(file) = file_uuid_to_metadata.get(&FileID(resource_id.to_string())) {
            // Verify file is actually in trash
            if file.restore_trash_prior_folder_path.is_none() {
                return Err("File is not in trash".to_string());
            }

            // Determine target restore location
            let target_folder = if let Some(restore_path) = &payload.restore_to_folder_path {
                // First try to find existing folder at the path
                let translation = translate_path_to_id(restore_path.clone());
                if let Some(existing_folder) = translation.folder {
                    existing_folder
                } else {
                    // Create the folder structure if it doesn't exist
                    let new_folder_uuid = ensure_folder_structure(
                        &restore_path.to_string(),
                        file.disk_id.clone(),
                        file.disk_type.clone(),
                        file.created_by.clone(),
                        file.canister_id.0.0.clone(),
                        file.has_sovereign_permissions.clone(),
                        None,
                        None,
                        None
                    );
                    
                    folder_uuid_to_metadata
                        .get(&new_folder_uuid)
                        .ok_or_else(|| "Failed to create restore folder path".to_string())?
                }
            } else {
                // Get the folder UUID from the stored path
                let path = file.restore_trash_prior_folder_path.clone().unwrap();
                let translation = translate_path_to_id(path.clone());
                if let Some(existing_folder) = translation.folder {
                    existing_folder
                } else {
                    // Create the folder structure if original path doesn't exist
                    let new_folder_uuid = ensure_folder_structure(
                        &path.to_string(),
                        file.disk_id.clone(),
                        file.disk_type.clone(),
                        file.created_by.clone(),
                        file.canister_id.0.0.clone(),
                        file.has_sovereign_permissions.clone(),
                        None,
                        None,
                        None
                    );
                    
                    folder_uuid_to_metadata
                        .get(&new_folder_uuid)
                        .ok_or_else(|| "Failed to create restore folder path".to_string())?
                }
            };

            // Verify target folder is not in trash
            if target_folder.restore_trash_prior_folder_path.is_some() {
                return Err(format!("Cannot restore to a folder that is in trash. Please first restore {}", target_folder.full_folder_path).to_string());
            }

            let file_id = FileID(resource_id.to_string());

            // Move file to target location
            let restored_file = move_file(
                &file_id,
                &target_folder,
                payload.file_conflict_resolution.clone(),
            )?;

            // Clear restore_trash_prior_folder_path
            file_uuid_to_metadata.with_mut(|map| {
                if let Some(file) = map.get_mut(&file_id) {
                    file.restore_trash_prior_folder_path = None;
                }
            });

            Ok(DirectoryActionResult::RestoreTrash(RestoreTrashResponse {
                restored_folders: Vec::new(),
                restored_files: vec![file_id],
            }))
        } else {
            Err("Resource not found in trash".to_string())
        }
    }


}