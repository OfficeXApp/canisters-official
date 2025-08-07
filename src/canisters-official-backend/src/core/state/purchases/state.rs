pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use ic_stable_structures::{memory_manager::MemoryId, BTreeMap, DefaultMemoryImpl, StableBTreeMap, StableVec};

    use crate::{
        core::{
            state::purchases::types::{Purchase, PurchaseID, PurchaseIDList}, types::UserID
        },
        debug_log, MEMORY_MANAGER,
    };

   
    use candid::CandidType;
    use serde::{Serialize, Deserialize};
    use ic_stable_structures::storable::Bound;
    use std::borrow::Cow;

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;

    // Unique MemoryId values for Purchase stable structures
    pub const PURCHASES_MEMORY_ID: MemoryId = MemoryId::new(53);
    pub const PURCHASES_BY_TIME_MEMORY_ID: MemoryId = MemoryId::new(54);
    pub const PURCHASES_BY_VENDOR_ID_MEMORY_ID: MemoryId = MemoryId::new(55);

    thread_local! {
        /// Stores Purchase records indexed by their unique PurchaseID.
        pub(crate) static PURCHASES_BY_ID_HASHTABLE: RefCell<StableBTreeMap<PurchaseID, Purchase, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(PURCHASES_MEMORY_ID))
            )
        );

        /// Stores a list of PurchaseIDs, ordered by creation time, for pagination.
        pub(crate) static PURCHASES_BY_TIME_LIST: RefCell<StableVec<PurchaseID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(PURCHASES_BY_TIME_MEMORY_ID))
            ).expect("Failed to initialize PURCHASES_BY_TIME_LIST")
        );

        pub(crate) static PURCHASES_BY_VENDOR_ID_HASHTABLE: RefCell<StableBTreeMap<UserID, PurchaseIDList, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(PURCHASES_BY_VENDOR_ID_MEMORY_ID))
            )
        );
    }

    impl ic_stable_structures::Storable for PurchaseIDList {
        const BOUND: Bound = Bound::Bounded {
            max_size: 256 * 1024, // Sufficiently large for a list of PurchaseIDs (e.g., 1024 IDs * 256 bytes/ID)
            is_fixed_size: false,
        };

        fn to_bytes(&self) -> Cow<[u8]> {
            let mut bytes = vec![];
            ciborium::ser::into_writer(self, &mut bytes)
                .expect("Failed to serialize StorablePurchaseIDVec");
            Cow::Owned(bytes)
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            ciborium::de::from_reader(bytes.as_ref())
                .expect("Failed to deserialize StorablePurchaseIDVec")
        }
    }

    /// Initializes all thread-local stable structures for Purchases.
    pub fn initialize() {
        // Force thread_locals in this module to initialize
        PURCHASES_BY_ID_HASHTABLE.with(|_| {});
        PURCHASES_BY_TIME_LIST.with(|_| {});
        PURCHASES_BY_VENDOR_ID_HASHTABLE.with(|_| {});
    }

    /// Adds a PurchaseID to the list associated with its vendor.
    pub fn add_purchase_to_vendor_list(vendor_id: &UserID, purchase_id: &PurchaseID) {
        PURCHASES_BY_VENDOR_ID_HASHTABLE.with(|map_ref| {
            let mut map = map_ref.borrow_mut();
            let mut purchase_ids = map.get(vendor_id).map_or_else(
                || PurchaseIDList { purchases: Vec::new() },
                |ids_vec| ids_vec,
            );

            if !purchase_ids.purchases.contains(purchase_id) {
                purchase_ids.purchases.push(purchase_id.clone());
                map.insert(vendor_id.clone(), purchase_ids);
            }
        });
    }

    /// Removes a PurchaseID from the list associated with its vendor.
    pub fn remove_purchase_from_vendor_list(vendor_id: &UserID, purchase_id: &PurchaseID) {
        PURCHASES_BY_VENDOR_ID_HASHTABLE.with(|map_ref| {
            let mut map = map_ref.borrow_mut();
            if let Some(mut purchase_ids) = map.remove(vendor_id) {
                purchase_ids.purchases.retain(|id| id != purchase_id);
                if !purchase_ids.purchases.is_empty() {
                    map.insert(vendor_id.clone(), purchase_ids);
                }
            }
        });
    }
}