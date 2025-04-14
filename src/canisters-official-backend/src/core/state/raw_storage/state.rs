// src/core/state/raw_storage/state.rs
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use crate::core::state::raw_storage::types::{ChunkId, FileChunk};

use super::types::{ChunkIdList, CHUNK_SIZE};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// Define memory IDs for different storage types
const CHUNKS_MEMORY_ID: MemoryId = MemoryId::new(1);
const FILE_CHUNKS_MEMORY_ID: MemoryId = MemoryId::new(2);
const FILE_META_MEMORY_ID: MemoryId = MemoryId::new(3);

// Implement Storable for our types
impl Storable for ChunkId {
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::into_writer(&self.0, &mut bytes).expect("Failed to serialize ChunkId");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let string: String = ciborium::from_reader(&bytes[..]).expect("Failed to deserialize ChunkId");
        ChunkId(string)
    }
}

impl Storable for FileChunk {
    const BOUND: Bound = Bound::Bounded {
        max_size: CHUNK_SIZE as u32 + 1024, // Base chunk size plus metadata overhead
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::into_writer(&self, &mut bytes).expect("Failed to serialize FileChunk");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ciborium::from_reader(&bytes[..]).expect("Failed to deserialize FileChunk")
    }
}

thread_local! {
    pub(crate) static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = 
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub(crate) static CHUNKS: RefCell<StableBTreeMap<ChunkId, FileChunk, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CHUNKS_MEMORY_ID))
        )
    );

    pub(crate) static FILE_CHUNKS: RefCell<StableBTreeMap<String, ChunkIdList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(FILE_CHUNKS_MEMORY_ID))
        )
    );

    pub(crate) static FILE_META: RefCell<StableBTreeMap<String, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(FILE_META_MEMORY_ID))
        )
    );
}

pub fn store_chunk(chunk: FileChunk) {
    CHUNKS.with(|chunks| {
        chunks.borrow_mut().insert(chunk.id.clone(), chunk.clone());
    });

    FILE_CHUNKS.with(|file_chunks| {
        let mut map = file_chunks.borrow_mut();
        let existing_chunks = map.get(&chunk.file_id)
            .map(|list| list.0.clone())
            .unwrap_or_default();
        let mut new_chunks = existing_chunks;
        new_chunks.push(chunk.id.clone());
        map.insert(chunk.file_id.clone(), ChunkIdList(new_chunks));
    });
}

pub fn get_chunk(chunk_id: &ChunkId) -> Option<FileChunk> {
    CHUNKS.with(|chunks| chunks.borrow().get(chunk_id))
}

pub fn get_file_chunks(file_id: &str) -> Vec<FileChunk> {
    FILE_CHUNKS.with(|file_chunks| {
        if let Some(chunk_list) = file_chunks.borrow().get(&file_id.to_string()) {
            chunk_list.0.iter()
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