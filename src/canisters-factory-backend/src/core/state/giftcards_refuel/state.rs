// src/core/state/drives/state.rs

pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_stable_structures::memory_manager::MemoryId;
    use ic_stable_structures::{DefaultMemoryImpl,StableBTreeMap,StableVec};

    use crate::core::state::giftcards_refuel::types::{FactoryRefuelHistoryRecord, GiftcardRefuelIDVec};
    use crate::core::state::giftcards_spawnorg::types::DriveID;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuelID;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuel;

    use crate::core::types::{UserID};
    use crate::MEMORY_MANAGER;
    

    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    
    // Define memory IDs for each storage
    pub const DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID_MEMORY_ID: MemoryId = MemoryId::new(5);
    pub const HISTORICAL_GIFTCARDS_REFUELS_MEMORY_ID: MemoryId = MemoryId::new(6);
    pub const DRIVE_TO_GIFTCARD_REFUEL_MEMORY_ID: MemoryId = MemoryId::new(7);
    pub const USER_TO_GIFTCARDS_REFUEL_MEMORY_ID: MemoryId = MemoryId::new(8);
    pub const GIFTCARD_REFUEL_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(9);

    thread_local! { 
        // GiftcardRefuel and deployment tracking
        pub(crate) static DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID: RefCell<StableBTreeMap<GiftcardRefuelID, FactoryRefuelHistoryRecord, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID_MEMORY_ID))
            )
        );
        
        pub(crate) static HISTORICAL_GIFTCARDS_REFUELS: RefCell<StableVec<GiftcardRefuelID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(HISTORICAL_GIFTCARDS_REFUELS_MEMORY_ID))
            ).expect("Failed to initialize HISTORICAL_GIFTCARDS_REFUELS")
        );
        
        pub(crate) static DRIVE_TO_GIFTCARD_REFUEL_HASHTABLE: RefCell<StableBTreeMap<DriveID, GiftcardRefuelID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DRIVE_TO_GIFTCARD_REFUEL_MEMORY_ID))
            )
        );
        
        // For the UserID to Vec<GiftcardRefuelID> mapping, we need a custom type similar to StringVec in the reference
        pub(crate) static USER_TO_GIFTCARDS_REFUEL_HASHTABLE: RefCell<StableBTreeMap<UserID, GiftcardRefuelIDVec, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(USER_TO_GIFTCARDS_REFUEL_MEMORY_ID))
            )
        );
        
        pub(crate) static GIFTCARD_REFUEL_BY_ID: RefCell<StableBTreeMap<GiftcardRefuelID, GiftcardRefuel, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(GIFTCARD_REFUEL_BY_ID_MEMORY_ID))
            )
        );
    }

    
    pub fn initialize() {
        // Force thread_locals in this module to initialize
        DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID.with(|_| {});
        HISTORICAL_GIFTCARDS_REFUELS.with(|_| {});
        DRIVE_TO_GIFTCARD_REFUEL_HASHTABLE.with(|_| {});
        USER_TO_GIFTCARDS_REFUEL_HASHTABLE.with(|_| {});
        GIFTCARD_REFUEL_BY_ID.with(|_| {});
    }
}