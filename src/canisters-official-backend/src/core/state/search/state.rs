// src/core/state/search/state.rs

pub mod state {
    use std::cell::RefCell;
    use std::collections::{HashMap, BTreeMap};
    use std::sync::Arc;

    use fst::{Map, MapBuilder, IntoStreamer, Streamer};
    use fst::automaton::Subsequence;

    use crate::core::api::permissions::directory::check_directory_permissions;
    use crate::core::api::permissions::system::check_system_permissions;
    use crate::core::state::directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata};
    use crate::core::state::directory::types::{DriveFullFilePath, FileID, FolderID};
    use crate::core::state::disks::types::DiskID;
    use crate::core::state::drives::state::state::{DRIVES_BY_ID_HASHTABLE, DRIVE_ID};
    use crate::core::state::drives::types::{DriveID, ExternalID};
    use crate::core::state::groups::types::GroupID;
    use crate::core::state::permissions::types::{DirectoryPermissionType, PermissionGranteeID, SystemPermissionType, SystemRecordIDEnum, SystemResourceID, SystemTableEnum};
    use crate::core::state::search::types::{SearchResult, SearchResultResourceID, SearchCategoryEnum};
    use crate::core::state::contacts::state::state::{CONTACTS_BY_ID_HASHTABLE};
    use crate::core::state::disks::state::state::{DISKS_BY_ID_HASHTABLE};
    use crate::core::state::groups::state::state::{GROUPS_BY_ID_HASHTABLE};
    use crate::core::types::{IDPrefix, UserID};
    use crate::rest::directory::types::DirectoryResourceID;
    

    // Thread-local storage for the FST search index
    thread_local! {
        static FST_INDEX: RefCell<Option<Arc<Map<Vec<u8>>>>> = RefCell::new(None);
        static PATH_TO_ID_MAP: RefCell<HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>> = RefCell::new(HashMap::new());
        static LAST_INDEX_UPDATE_MS: RefCell<u64> = RefCell::new(0);
    }
    
    /// Builds or rebuilds the search index for the entire drive
    /// This is the primary function to call when you need to create or update the index
    pub fn reindex_drive() -> Result<usize, String> {
        // Get current time in milliseconds
        let current_time_ms = ic_cdk::api::time() / 1_000_000; // Convert nanoseconds to milliseconds
        
        // Index all resources
        let result = build_index();
        
        // Update the last index time if successful
        if let Ok(count) = result {
            // Update thread-local timestamp
            LAST_INDEX_UPDATE_MS.with(|cell| {
                *cell.borrow_mut() = current_time_ms;
            });
            
            // Update the Drive record to store the last_indexed_ms value
            DRIVE_ID.with(|drive_id| {
                let drive_id_val = drive_id.clone(); // a copy of the key
                DRIVES_BY_ID_HASHTABLE.with(|drives| {
                    let mut map_ref = drives.borrow_mut();
                    
                    // Fetch a Drive (clone) from the stable map:
                    if let Some(mut drive) = map_ref.get(&drive_id_val) {
                        // Update the Drive in regular memory
                        drive.last_indexed_ms = Some(current_time_ms);
            
                        // Reinsert into the stable map
                        map_ref.insert(drive_id_val, drive);
                    }
                });
            });

        }
        
        result
    }
    
    /// Internal function that builds the index from all resources
    /// This handles the actual FST construction
    fn build_index() -> Result<usize, String> {
        let mut builder = MapBuilder::memory();
        let mut entries = BTreeMap::new();
        let mut path_to_id = HashMap::new();
    
        // Index files
        index_files(&mut entries, &mut path_to_id);
        
        // Index folders
        index_folders(&mut entries, &mut path_to_id);
        
        // Index contacts
        index_contacts(&mut entries, &mut path_to_id);
        
        // Index disks
        index_disks(&mut entries, &mut path_to_id);
        
        // Index drives
        index_drives(&mut entries, &mut path_to_id);
        
        // Index groups
        index_groups(&mut entries, &mut path_to_id);
    
        // Get the total count of indexed items
        let indexed_count = entries.len();
    
        // Build the FST Map
        for (key, value) in &entries {
            if let Err(e) = builder.insert(key, *value) {
                return Err(format!("Failed to build search index: {}", e));
            }
        }
    
        // Finish building and store in thread-local storage
        let fst_map = builder.into_map();
        let arc_map = Arc::new(fst_map);
        
        FST_INDEX.with(|cell| {
            *cell.borrow_mut() = Some(arc_map);
        });
        
        PATH_TO_ID_MAP.with(|cell| {
            *cell.borrow_mut() = path_to_id;
        });
        
        Ok(indexed_count)
    }

    /// Index files
    fn index_files(entries: &mut BTreeMap<String, u64>, path_to_id: &mut HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>) {
        file_uuid_to_metadata.with(|map| {
            for (file_id, metadata) in map.iter() {
                // Skip deleted files
                if !metadata.deleted {
                    // Normalize the path for search
                    let normalized = normalize_path(&metadata.full_directory_path.0);
                    
                    path_to_id.insert(normalized.clone(), (
                        SearchResultResourceID::File(file_id.clone()),
                        SearchCategoryEnum::Files
                    ));
                    
                    // Insert with a default score of 1
                    entries.insert(normalized, 1u64);
                }
            }
        });
    }

    /// Index folders
    fn index_folders(entries: &mut BTreeMap<String, u64>, path_to_id: &mut HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>) {
        folder_uuid_to_metadata.with(|map| {
            for (folder_id, metadata) in map.iter() {
                // Skip deleted folders
                if !metadata.deleted {
                    // Normalize the path for search
                    let normalized = normalize_path(&metadata.full_directory_path.0);
                    
                    path_to_id.insert(normalized.clone(), (
                        SearchResultResourceID::Folder(folder_id.clone()),
                        SearchCategoryEnum::Folders
                    ));
                    
                    // Insert with a default score of 1
                    entries.insert(normalized, 1u64);
                }
            }
        });
    }

    /// Index contacts
    fn index_contacts(entries: &mut BTreeMap<String, u64>, path_to_id: &mut HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>) {
        CONTACTS_BY_ID_HASHTABLE.with(|contacts| {
            for (contact_id, contact) in contacts.borrow().iter() {
                // Create a searchable string with all contact fields
                let search_string = format!(
                    "{}|{}|{}|{}",
                    contact_id.0,
                    contact.name,
                    contact.icp_principal.0.0,
                    contact.evm_public_address
                );
                
                // Normalize for search
                let normalized = normalize_path(&search_string);
                
                path_to_id.insert(normalized.clone(), (
                    SearchResultResourceID::Contact(contact_id.clone()),
                    SearchCategoryEnum::Contacts
                ));
                
                // Insert with a default score of 1
                entries.insert(normalized, 1u64);
            }
        });
    }

    /// Index disks
    fn index_disks(entries: &mut BTreeMap<String, u64>, path_to_id: &mut HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>) {
        DISKS_BY_ID_HASHTABLE.with(|disks| {
            for (disk_id, disk) in disks.borrow().iter() {
                // Create a searchable string with disk id, name, and external_id
                let search_string = format!(
                    "{}|{}|{}",
                    disk_id.0,
                    disk.name,
                    disk.external_id.clone().unwrap_or(ExternalID("".to_string()))
                );
                
                // Normalize for search
                let normalized = normalize_path(&search_string);
                
                path_to_id.insert(normalized.clone(), (
                    SearchResultResourceID::Disk(disk_id.clone()),
                    SearchCategoryEnum::Disks
                ));
                
                // Insert with a default score of 1
                entries.insert(normalized, 1u64);
            }
        });
    }

    /// Index drives
    fn index_drives(entries: &mut BTreeMap<String, u64>, path_to_id: &mut HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>) {
        DRIVES_BY_ID_HASHTABLE.with(|drives| {
            for (drive_id, drive) in drives.borrow().iter() {
                // Create a searchable string with drive fields
                let search_string = format!(
                    "{}|{}|{}|{}",
                    drive_id.0,
                    drive.name,
                    drive.icp_principal.0.0,
                    drive.endpoint_url.0
                );
                
                // Normalize for search
                let normalized = normalize_path(&search_string);
                
                path_to_id.insert(normalized.clone(), (
                    SearchResultResourceID::Drive(drive_id.clone()),
                    SearchCategoryEnum::Drives
                ));
                
                // Insert with a default score of 1
                entries.insert(normalized, 1u64);
            }
        });
    }

    /// Index groups
    fn index_groups(entries: &mut BTreeMap<String, u64>, path_to_id: &mut HashMap<String, (SearchResultResourceID, SearchCategoryEnum)>) {
        GROUPS_BY_ID_HASHTABLE.with(|groups| {
            for (group_id, group) in groups.borrow().iter() {
                // Create a searchable string with group id, name, and drive_id
                let search_string = format!(
                    "{}|{}|{}",
                    group_id.0,
                    group.name,
                    group.drive_id.0
                );
                
                // Normalize for search
                let normalized = normalize_path(&search_string);
                
                path_to_id.insert(normalized.clone(), (
                    SearchResultResourceID::Group(group_id.clone()),
                    SearchCategoryEnum::Groups
                ));
                
                // Insert with a default score of 1
                entries.insert(normalized, 1u64);
            }
        });
    }

    /// Search the index with fuzzy matching and return results sorted by relevance
    /// Now supports filtering by categories
    pub fn raw_query(query: &str, _max_edit_distance: u32, categories: Option<Vec<SearchCategoryEnum>>) -> Vec<SearchResult> {
        // Early return if index isn't built yet
        let index_option = FST_INDEX.with(|cell| cell.borrow().clone());
        let index = match index_option {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        
        // Normalize the query
        let normalized_query = normalize_for_query(query);
        
        // Use subsequence matching (a simpler form of fuzzy matching)
        let subseq = Subsequence::new(&normalized_query);
        
        // Search the FST
        let mut stream = index.search(subseq).into_stream();
        let mut matches = Vec::new();
        
        while let Some((path_bytes, score)) = stream.next() {
            if let Ok(path) = String::from_utf8(path_bytes.to_vec()) {
                let resource_info = PATH_TO_ID_MAP.with(|cell| {
                    cell.borrow().get(&path).cloned()
                });
                
                if let Some((resource_id, category)) = resource_info {
                    // Filter by category if specified
                    if let Some(ref filter_categories) = categories {
                        if !filter_categories.contains(&SearchCategoryEnum::All) && 
                           !filter_categories.contains(&category) {
                            continue;
                        }
                    }
                    
                    // Generate title and preview based on resource type
                    let (title, preview, created_at, updated_at, metadata) = generate_title_and_preview(&resource_id);
                    
                    matches.push(SearchResult {
                        title,
                        preview,
                        score,
                        resource_id: resource_id.to_string(),
                        category,
                        created_at,
                        updated_at,
                        metadata,
                    });
                }
            }
        }
        
        // Sort by score (higher score = better match)
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        
        matches
    }

    pub async fn filter_search_results_by_permission(
        results: &[SearchResult], 
        grantee_id: &PermissionGranteeID, 
        is_owner: bool
    ) -> Vec<SearchResult> {
        let mut filtered_results = Vec::new();
        
        // Owners see everything, bypass permission checks
        if is_owner {
            return results.to_vec();
        }
        
        for result in results {
            let has_permission = match &result.category {
                // Directory resources (files and folders)
                SearchCategoryEnum::Files => {
                    if let SearchResultResourceID::File(file_id) = &SearchResultResourceID::File(FileID(result.resource_id.clone())) {
                        let resource_id = DirectoryResourceID::File(file_id.clone());
                        let permissions = check_directory_permissions(
                            resource_id.clone(),
                            grantee_id.clone()
                        ).await;
                        permissions.contains(&DirectoryPermissionType::View)
                    } else {
                        false // This should not happen based on category
                    }
                },
                SearchCategoryEnum::Folders => {
                    if let SearchResultResourceID::Folder(folder_id) = &SearchResultResourceID::Folder(FolderID(result.resource_id.clone())) {
                        let resource_id = DirectoryResourceID::Folder(folder_id.clone());
                        let permissions = check_directory_permissions(
                            resource_id.clone(),
                            grantee_id.clone()
                        ).await;
                        permissions.contains(&DirectoryPermissionType::View)
                    } else {
                        false // This should not happen based on category
                    }
                },
                
                // System resources
                SearchCategoryEnum::Contacts => {
                    if let SearchResultResourceID::Contact(user_id) = &SearchResultResourceID::Contact(UserID(result.resource_id.clone())) {
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(user_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        // Check table-wide permission if no specific permission found
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Contacts),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else {
                        false // This should not happen based on category
                    }
                },
                SearchCategoryEnum::Disks => {
                    if let SearchResultResourceID::Disk(disk_id) = &SearchResultResourceID::Disk(DiskID(result.resource_id.clone())) {
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        // Check table-wide permission if no specific permission found
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Disks),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else {
                        false // This should not happen based on category
                    }
                },
                SearchCategoryEnum::Drives => {
                    if let SearchResultResourceID::Drive(drive_id) = &SearchResultResourceID::Drive(DriveID(result.resource_id.clone())) {
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Drive(drive_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        // Check table-wide permission if no specific permission found
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Drives),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else {
                        false // This should not happen based on category
                    }
                },
                SearchCategoryEnum::Groups => {
                    if let SearchResultResourceID::Group(group_id) = &SearchResultResourceID::Group(GroupID(result.resource_id.clone())) {
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Group(group_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        // Check table-wide permission if no specific permission found
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Groups),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else {
                        false // This should not happen based on category
                    }
                },
                // Handle the All category by checking the specific resource type
                SearchCategoryEnum::All => {
                    // We need to determine the type of resource from the category or some other way
                    // Since we can't directly match on result.resource_id as an enum, we need to use the category to guide our handling
                    
                    // First, try to infer the resource type from the ID format or metadata
                    // This is a simplified approach - you might need a more robust way to determine the resource type
                    if result.resource_id.starts_with(IDPrefix::File.as_str()) {
                        // Handle as file
                        let file_id = FileID(result.resource_id.clone());
                        let resource_id = DirectoryResourceID::File(file_id);
                        let permissions = check_directory_permissions(
                            resource_id,
                            grantee_id.clone()
                        ).await;
                        permissions.contains(&DirectoryPermissionType::View)
                    } else if result.resource_id.starts_with(IDPrefix::Folder.as_str()) {
                        // Handle as folder
                        let folder_id = FolderID(result.resource_id.clone());
                        let resource_id = DirectoryResourceID::Folder(folder_id);
                        let permissions = check_directory_permissions(
                            resource_id,
                            grantee_id.clone()
                        ).await;
                        permissions.contains(&DirectoryPermissionType::View)
                    } else if result.resource_id.starts_with(IDPrefix::User.as_str()) {
                        // Handle as contact
                        let user_id = UserID(result.resource_id.clone());
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::User(user_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Contacts),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else if result.resource_id.starts_with(IDPrefix::Disk.as_str()) {
                        // Handle as disk
                        let disk_id = DiskID(result.resource_id.clone());
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Disk(disk_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Disks),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else if result.resource_id.starts_with(IDPrefix::Drive.as_str()) {
                        // Handle as drive
                        let drive_id = DriveID(result.resource_id.clone());
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Drive(drive_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Drives),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else if result.resource_id.starts_with(IDPrefix::Group.as_str()) {
                        // Handle as group
                        let group_id = GroupID(result.resource_id.clone());
                        let resource_id = SystemResourceID::Record(SystemRecordIDEnum::Group(group_id.0.clone()));
                        let permissions = check_system_permissions(
                            resource_id,
                            grantee_id.clone()
                        );
                        if !permissions.contains(&SystemPermissionType::View) {
                            let table_permission = check_system_permissions(
                                SystemResourceID::Table(SystemTableEnum::Groups),
                                grantee_id.clone()
                            );
                            table_permission.contains(&SystemPermissionType::View)
                        } else {
                            true
                        }
                    } else {
                        // Unknown resource type
                        false
                    }
                }
            };
            
            if has_permission {
                filtered_results.push(result.clone());
            }
        }
        
        filtered_results
    }
    
    /// Helper function to generate title and preview for each resource type
    fn generate_title_and_preview(resource_id: &SearchResultResourceID) -> (String, String, u64, u64, Option<String>) {
        match resource_id {
            SearchResultResourceID::File(file_id) => {
                let mut title = String::new();
                let mut preview = String::new();
                let mut created_at = 0;
                let mut updated_at = 0;
                let mut result_metadata: Option<String> = None;
                
                file_uuid_to_metadata.with(|map| {
                    if let Some(metadata) = map.get(file_id) {
                        // Extract filename from path
                        let path_parts: Vec<&str> = metadata.full_directory_path.0.split('/').collect();
                        title = path_parts.last().unwrap_or(&"").to_string();
                    
                        if path_parts.len() >= 2 {
                            // Get parent folder (second-to-last element) and filename (last element)
                            let parent = path_parts.get(path_parts.len() - 2).unwrap_or(&"");
                            let filename = path_parts.last().unwrap_or(&"");
                            preview = format!("{}/{}", parent, filename);
                        } else {
                            // If there's no parent folder, just use the filename
                            preview = title.clone();
                        }

                        created_at = metadata.created_at;
                        updated_at = metadata.last_updated_date_ms;
                        result_metadata = Some(format!("/{}/{}/{}", metadata.disk_type.clone(), metadata.disk_id.clone(), metadata.id.clone()));
                    }
                });
                
                (title, preview, created_at, updated_at, result_metadata)
            },
            SearchResultResourceID::Folder(folder_id) => {
                let mut title = String::new();
                let mut preview = String::new();
                let mut created_at = 0;
                let mut updated_at = 0;
                let mut result_metadata: Option<String> = None;
                
                folder_uuid_to_metadata.with(|map| {
                    if let Some(metadata) = map.get(folder_id) {
                        title = metadata.name.clone();
                        
                        let path_parts: Vec<&str> = metadata.full_directory_path.0.split('/').collect();
                        if path_parts.len() >= 2 {
                            // Get parent folder (second-to-last element) and current folder name (last element)
                            let parent = path_parts.get(path_parts.len() - 2).unwrap_or(&"");
                            let folder_name = path_parts.last().unwrap_or(&"");
                            preview = format!("{}/{}", parent, folder_name);
                        } else {
                            // If there's no parent folder, just use the folder name
                            preview = title.clone();
                        }

                        created_at = metadata.created_at;
                        updated_at = metadata.last_updated_date_ms;
                        result_metadata = Some(format!("/{}/{}/{}", metadata.disk_type.clone(), metadata.disk_id.clone(), metadata.id.clone()));
                    }
                });
                
                (title, preview, created_at, updated_at, result_metadata)
            },
            SearchResultResourceID::Contact(user_id) => {
                let mut title = String::new();
                let mut preview = String::new();
                let mut created_at = 0;
                let mut updated_at = 0;
                
                CONTACTS_BY_ID_HASHTABLE.with(|contacts| {
                    if let Some(contact) = contacts.borrow().get(user_id) {
                        title = contact.name.clone();
                        preview = contact.icp_principal.0.0.clone();
                        created_at = contact.created_at;
                        updated_at = contact.last_online_ms;
                    }
                });
                
                (title, preview, created_at, updated_at, None)
            },
            SearchResultResourceID::Disk(disk_id) => {
                let mut title = String::new();
                let mut preview = String::new();
                let mut created_at = 0;
                let mut updated_at = 0;
                
                DISKS_BY_ID_HASHTABLE.with(|disks| {
                    if let Some(disk) = disks.borrow().get(disk_id) {
                        title = disk.name.clone();
                        preview = disk.external_id.clone().unwrap_or(ExternalID("".to_string())).0;
                        created_at = disk.created_at;
                        updated_at = disk.created_at;
                    }
                });
                
                (title, preview, created_at, updated_at, None)
            },
            SearchResultResourceID::Drive(drive_id) => {
                let mut title = String::new();
                let mut preview = String::new();
                let mut created_at = 0;
                let mut updated_at = 0;
                
                DRIVES_BY_ID_HASHTABLE.with(|drives| {
                    if let Some(drive) = drives.borrow().get(drive_id) {
                        title = drive.name.clone();
                        preview = drive.icp_principal.0.0.clone();
                        created_at = drive.created_at;
                        updated_at = drive.created_at;
                    }
                });
                
                (title, preview, created_at, updated_at, None)
            },
            SearchResultResourceID::Group(group_id) => {
                let mut title = String::new();
                let mut preview = String::new();
                let mut created_at = 0;
                let mut updated_at = 0;
                
                GROUPS_BY_ID_HASHTABLE.with(|groups| {
                    if let Some(group) = groups.borrow().get(group_id) {
                        title = group.name.clone();
                        preview = group.drive_id.0.clone();
                        created_at = group.created_at;
                        updated_at = group.last_modified_at;
                    }
                });
                
                (title, preview, created_at, updated_at, None)
            },
        }
    }

    /// Get the timestamp (in milliseconds) of when the index was last updated
    pub fn get_last_index_update_time() -> u64 {
        LAST_INDEX_UPDATE_MS.with(|cell| *cell.borrow())
    }

    /// Helper function to normalize a path for indexing
    fn normalize_path(path: &str) -> String {
        path.to_lowercase()
            .trim()
            .replace("//", "/")
    }

    /// Helper function to normalize a query for searching
    /// This should match the same normalization approach used for indexed paths
    fn normalize_for_query(query: &str) -> String {
        query.to_lowercase()
            .trim()
            .replace(" ", "") // Remove spaces for more flexible matching
    }
}