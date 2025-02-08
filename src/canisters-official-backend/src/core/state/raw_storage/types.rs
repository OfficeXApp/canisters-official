// src/core/state/raw_storage/types.rs
use candid::{CandidType, Decode, Encode}; 
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

pub const CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2MB chunks

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct ChunkId(pub String);

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Storable for ChunkId {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap()) 
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct FileChunk {
    pub id: ChunkId,
    pub file_id: String,
    pub chunk_index: u32,
    pub data: Vec<u8>,
    pub size: usize
}

impl Storable for FileChunk {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}