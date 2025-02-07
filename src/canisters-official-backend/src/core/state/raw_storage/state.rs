// src/core/state/raw_storage/state.rs
use std::cell::RefCell;
use std::collections::HashMap;
use crate::core::state::raw_storage::types::{ChunkId, FileChunk};

thread_local! {
    pub static CHUNKS: RefCell<HashMap<ChunkId, FileChunk>> = RefCell::new(HashMap::new());
    pub static FILE_CHUNKS: RefCell<HashMap<String, Vec<ChunkId>>> = RefCell::new(HashMap::new());
    pub static FILE_META: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub fn store_chunk(chunk: FileChunk) {
    CHUNKS.with(|chunks| {
        chunks.borrow_mut().insert(chunk.id.clone(), chunk.clone());
    });

    FILE_CHUNKS.with(|file_chunks| {
        let mut map = file_chunks.borrow_mut();
        map.entry(chunk.file_id.clone())
           .or_insert_with(Vec::new)
           .push(chunk.id);
    });
}

pub fn get_chunk(chunk_id: &ChunkId) -> Option<FileChunk> {
    CHUNKS.with(|chunks| chunks.borrow().get(chunk_id).cloned())
}

pub fn get_file_chunks(file_id: &str) -> Vec<FileChunk> {
    FILE_CHUNKS.with(|file_chunks| {
        if let Some(chunk_ids) = file_chunks.borrow().get(file_id) {
            chunk_ids.iter()
                    .filter_map(|id| get_chunk(id))
                    .collect()
        } else {
            Vec::new()
        }
    })
}


pub fn store_filename(file_id: &str, filename: &str) {
    FILE_META.with(|fmeta| {
        fmeta.borrow_mut().insert(file_id.to_string(), filename.to_string());
    });
}