pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, BTreeMap, DefaultMemoryImpl, StableBTreeMap, StableVec};

    use crate::{
        core::{
            state::job_runs::types::{JobRun, JobRunID, JobRunIDList}, types::UserID
        },
        debug_log, MEMORY_MANAGER,
    };

   
    use candid::CandidType;
    use serde::{Serialize, Deserialize};
    use ic_stable_structures::storable::Bound;
    use std::borrow::Cow;

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

    // Unique MemoryId values for JobRun stable structures
    pub const JOB_RUNS_MEMORY_ID: MemoryId = MemoryId::new(13);
    pub const JOB_RUNS_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(14);
    pub const JOB_RUNS_BY_VENDOR_ID_MEMORY_ID: MemoryId = MemoryId::new(15);

    thread_local! {
        /// Stores JobRun records indexed by their unique JobRunID.
        pub(crate) static JOB_RUNS_BY_ID_HASHTABLE: RefCell<StableBTreeMap<JobRunID, JobRun, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(JOB_RUNS_MEMORY_ID))
            )
        );

        /// Stores a list of JobRunIDs, ordered by creation time, for pagination.
        pub(crate) static JOB_RUNS_BY_TIME_LIST: RefCell<StableVec<JobRunID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(JOB_RUNS_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize JOB_RUNS_BY_TIME_LIST")
        );

        pub(crate) static JOB_RUNS_BY_VENDOR_ID_HASHTABLE: RefCell<StableBTreeMap<UserID, JobRunIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(JOB_RUNS_BY_VENDOR_ID_MEMORY_ID))
            )
        );
    }

    impl ic_stable_structures::Storable for JobRunIDList {
        const BOUND: Bound = Bound::Bounded {
            max_size: 256 * 1024, // Sufficiently large for a list of JobRunIDs (e.g., 1024 IDs * 256 bytes/ID)
            is_fixed_size: false,
        };

        fn to_bytes(&self) -> Cow<[u8]> {
            let mut bytes = vec![];
            ciborium::ser::into_writer(self, &mut bytes)
                .expect("Failed to serialize StorableJobRunIDVec");
            Cow::Owned(bytes)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            ciborium::de::from_reader(bytes.as_ref())
                .expect("Failed to deserialize StorableJobRunIDVec")
        }
    }

    /// Initializes all thread-local stable structures for JobRuns.
    pub fn initialize() {
        // Force thread_locals in this module to initialize
        JOB_RUNS_BY_ID_HASHTABLE.with(|_| {});
        JOB_RUNS_BY_TIME_LIST.with(|_| {});
        JOB_RUNS_BY_VENDOR_ID_HASHTABLE.with(|_| {});
    }

    /// Adds a JobRunID to the list associated with its vendor.
    pub fn add_job_run_to_vendor_list(vendor_id: &UserID, job_run_id: &JobRunID) {
        JOB_RUNS_BY_VENDOR_ID_HASHTABLE.with(|map_ref| {
            let mut map = map_ref.borrow_mut();
            let mut job_run_ids = map.get(vendor_id).map_or_else(
                || JobRunIDList { job_runs: Vec::new() },
                |ids_vec| ids_vec,
            );

            if !job_run_ids.job_runs.contains(job_run_id) {
                job_run_ids.job_runs.push(job_run_id.clone());
                map.insert(vendor_id.clone(), job_run_ids);
            }
        });
    }

    /// Removes a JobRunID from the list associated with its vendor.
    pub fn remove_job_run_from_vendor_list(vendor_id: &UserID, job_run_id: &JobRunID) {
        JOB_RUNS_BY_VENDOR_ID_HASHTABLE.with(|map_ref| {
            let mut map = map_ref.borrow_mut();
            if let Some(mut job_run_ids) = map.remove(vendor_id) {
                job_run_ids.job_runs.retain(|id| id != job_run_id);
                if !job_run_ids.job_runs.is_empty() {
                    map.insert(vendor_id.clone(), job_run_ids);
                }
            }
        });
    }
}