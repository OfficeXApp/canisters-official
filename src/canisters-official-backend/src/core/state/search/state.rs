// src/core/state/search/state.rs

pub mod state {
    use std::cell::RefCell;
    use std::collections::{HashMap, BTreeMap};
    use std::sync::Arc;

    use fst::{Map, MapBuilder, IntoStreamer, Streamer};
    use fst::automaton::Subsequence;

    use crate::core::state::directory::state::state::{file_uuid_to_metadata, folder_uuid_to_metadata};
    use crate::core::state::directory::types::{DriveFullFilePath, FileUUID, FolderUUID};
    use crate::core::state::drives::state::state::{DRIVES_BY_ID_HASHTABLE, DRIVE_ID};
    // Removed unused IDPrefix import

    // Define a SearchResultResourceID enum to store either a FileUUID or FolderUUID
    #[derive(Debug, Clone)]
    pub enum SearchResultResourceID {
        File(FileUUID),
        Folder(FolderUUID),
    }

    // Thread-local storage for the FST search index
    thread_local! {
        static FST_INDEX: RefCell<Option<Arc<Map<Vec<u8>>>>> = RefCell::new(None);
        static PATH_TO_ID_MAP: RefCell<HashMap<String, SearchResultResourceID>> = RefCell::new(HashMap::new());
        static LAST_INDEX_UPDATE_MS: RefCell<u64> = RefCell::new(0);
    }

    #[derive(Debug, Clone)]
    pub struct SearchResult {
        pub path: String,
        pub score: u64,
        pub resource_id: SearchResultResourceID,
    }

    /// Builds or rebuilds the search index for the entire drive
    /// This is the primary function to call when you need to create or update the index
    pub fn reindex_drive() -> Result<(), String> {
        // Get current time in milliseconds
        let current_time_ms = ic_cdk::api::time() / 1_000_000; // Convert nanoseconds to milliseconds
        
        // Collect all paths from files and folders
        let mut paths = Vec::new();
        
        // Collect file paths
        file_uuid_to_metadata.with(|map| {
            for (file_id, metadata) in map.iter() {
                // Skip deleted files
                if !metadata.deleted {
                    paths.push(metadata.full_file_path.clone());
                }
            }
        });
        
        // Collect folder paths
        folder_uuid_to_metadata.with(|map| {
            for (folder_id, metadata) in map.iter() {
                // Skip deleted folders
                if !metadata.deleted {
                    paths.push(metadata.full_folder_path.clone());
                }
            }
        });
        
        // Build the index with the collected paths
        let result = build_index_with_paths(paths);
        
        // Update the last index time if successful
        if result.is_ok() {
            // Update thread-local timestamp
            LAST_INDEX_UPDATE_MS.with(|cell| {
                *cell.borrow_mut() = current_time_ms;
            });
            
            // Update the Drive record to store the last_indexed_ms value
            DRIVE_ID.with(|drive_id| {
                DRIVES_BY_ID_HASHTABLE.with(|drives| {
                    if let Some(drive) = drives.borrow_mut().get_mut(drive_id) {
                        drive.last_indexed_ms = Some(current_time_ms);
                    }
                });
            });
        }
        
        result
    }
    
    /// Internal function that builds the index from a list of paths
    /// This handles the actual FST construction
    fn build_index_with_paths(paths: Vec<DriveFullFilePath>) -> Result<(), String> {
        let mut builder = MapBuilder::memory();
        let mut entries = BTreeMap::new();
        let mut path_to_id = HashMap::new();

        // Prepare the entries for FST
        for path in paths {
            // Normalize the path for search
            let normalized = normalize_path(&path.0);
            
            // Check if path belongs to a file
            let file_id = file_uuid_to_metadata.with(|map| {
                map.iter().find(|(_, metadata)| metadata.full_file_path.0 == path.0)
                   .map(|(file_id, _)| file_id.clone())
            });
            
            if let Some(file_id) = file_id {
                path_to_id.insert(normalized.clone(), SearchResultResourceID::File(file_id));
                // Insert with a default score of 1
                entries.insert(normalized, 1u64);
                continue;
            }
            
            // Check if path belongs to a folder
            let folder_id = folder_uuid_to_metadata.with(|map| {
                map.iter().find(|(_, metadata)| metadata.full_folder_path.0 == path.0)
                   .map(|(folder_id, _)| folder_id.clone())
            });
            
            if let Some(folder_id) = folder_id {
                path_to_id.insert(normalized.clone(), SearchResultResourceID::Folder(folder_id));
                // Insert with a default score of 1
                entries.insert(normalized, 1u64);
            }
        }

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
        
        Ok(())
    }

    /// Search the index with fuzzy matching and return results sorted by relevance
    pub fn search(query: &str, _max_edit_distance: u32, limit: usize) -> Vec<SearchResult> {
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
                let resource_id = PATH_TO_ID_MAP.with(|cell| {
                    cell.borrow().get(&path).cloned()
                });
                
                if let Some(resource_id) = resource_id {
                    matches.push(SearchResult {
                        path,
                        score,
                        resource_id,
                    });
                }
                
                // Limit the number of results
                if matches.len() >= limit {
                    break;
                }
            }
        }
        
        // Sort by score (higher score = better match)
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        
        matches
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