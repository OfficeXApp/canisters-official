use ic_stable_structures::{storable::Bound, Storable};
// src/core/state/raw_storage/types.rs
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use std::{borrow::Cow, fmt};


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkIdList(pub Vec<ChunkId>);

impl Storable for ChunkIdList {
    const BOUND: Bound = Bound::Bounded {
        max_size: 1024 * 1024, // Adjust based on your needs
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::into_writer(&self.0, &mut bytes).expect("Failed to serialize ChunkIdList");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let vec: Vec<ChunkId> = ciborium::from_reader(&bytes[..])
            .expect("Failed to deserialize ChunkIdList");
        ChunkIdList(vec)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SerdeDiff)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UploadStatus {
    Queued,     // File is created but no chunks uploaded yet
    Pending,    // Some chunks uploaded, not completed
    Completed,  // All chunks uploaded and verified
}