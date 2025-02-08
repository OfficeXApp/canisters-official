// src/core/state/raw_storage/state.rs
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, 
    StableBTreeMap,
    Memory
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use crate::core::state::raw_storage::types::{ChunkId, FileChunk};

const CHUNKS_MEM_ID: MemoryId = MemoryId::new(1);
const FILE_CHUNKS_MEM_ID: MemoryId = MemoryId::new(2);
const FILE_META_MEM_ID: MemoryId = MemoryId::new(3);

thread_local! {
    // Initialize the memory manager
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // Store chunks in stable memory using a different initialization approach
    static STABLE_CHUNKS: RefCell<StableBTreeMap<ChunkId, Vec<u8>, DefaultMemoryImpl>> = RefCell::new(
        StableBTreeMap::new(DefaultMemoryImpl::default())
    );

    // Store chunk metadata in heap memory
    static CHUNKS: RefCell<BTreeMap<ChunkId, FileChunk>> = RefCell::new(BTreeMap::new());
    pub(crate) static FILE_CHUNKS: RefCell<BTreeMap<String, Vec<ChunkId>>> = RefCell::new(BTreeMap::new());
    pub(crate) static FILE_META: RefCell<BTreeMap<String, String>> = RefCell::new(BTreeMap::new());
}

pub fn store_chunk(chunk: FileChunk) {
    STABLE_CHUNKS.with(|chunks| {
        chunks.borrow_mut().insert(chunk.id.clone(), chunk.data.clone());
    });

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
            // Get chunks using the stored chunk IDs
            chunk_ids.iter()
                    .filter_map(|chunk_id| {
                        CHUNKS.with(|chunks| {
                            chunks.borrow().get(chunk_id).cloned()
                        })
                    })
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

pub fn get_filename(file_id: &str) -> Option<String> {
    FILE_META.with(|fmeta| {
        fmeta.borrow().get(file_id).cloned()
    })
}