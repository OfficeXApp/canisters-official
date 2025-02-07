
pub mod drive_internals {
    use crate::{
        core::{api::uuid::generate_unique_id, state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata, full_file_path_to_uuid, full_folder_path_to_uuid}, types::{DriveFullFilePath, FolderMetadata, FolderUUID}}, disks::types::DiskTypeEnum}, types::{ICPPrincipalString, PublicKeyBLS, UserID}}, debug_log, 
        
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

    pub fn ensure_root_folder(storage_location: &DiskTypeEnum, user_id: &UserID) -> FolderUUID {
        let root_path = DriveFullFilePath(format!("{}::", storage_location.to_string()));
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
                canister_id: ICPPrincipalString(PublicKeyBLS(ic_cdk::api::id().to_text())),
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
}