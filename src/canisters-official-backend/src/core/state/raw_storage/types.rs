// src/core/state/raw_storage/types.rs
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId(pub String);

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub id: ChunkId,
    pub file_id: String,
    pub chunk_index: u32,
    pub data: Vec<u8>,
    pub size: usize
}

pub const CHUNK_SIZE: usize = 3 * 1024 * (1024 / 2); // 1.5MB chunks