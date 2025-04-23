// src/core/state/drives/state.rs

pub mod state {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use ic_stable_structures::memory_manager::MemoryId;
    use ic_stable_structures::{StableBTreeMap,StableVec};
    use ic_stable_structures::StableCell;
    use ic_stable_structures::DefaultMemoryImpl;

    use crate::core::api::helpers::get_appropriate_url_endpoint;
    use crate::core::state::giftcards_spawnorg::types::DriveID;
    use crate::core::state::giftcards_spawnorg::types::DriveRESTUrlEndpoint;
    use crate::core::state::giftcards_spawnorg::types::FactorySpawnHistoryRecord;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrgID;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrg;

    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrgIDVec;
    use crate::core::types::{UserID,PublicKeyICP};
    use crate::debug_log;
    use crate::MEMORY_MANAGER;


    type Memory = ic_stable_structures::memory_manager::VirtualMemory<DefaultMemoryImpl>;
    
    // Define memory IDs for each storage
    pub const VERSION_MEMORY_ID: MemoryId = MemoryId::new(10);
    pub const OWNER_ID_MEMORY_ID: MemoryId = MemoryId::new(11);
    pub const URL_ENDPOINT_MEMORY_ID: MemoryId = MemoryId::new(12);
    pub const DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID_MEMORY_ID: MemoryId = MemoryId::new(13);
    pub const HISTORICAL_GIFTCARDS_SPAWNORGS_MEMORY_ID: MemoryId = MemoryId::new(14);
    pub const DRIVE_TO_GIFTCARD_SPAWNORG_MEMORY_ID: MemoryId = MemoryId::new(15);
    pub const USER_TO_GIFTCARDS_SPAWNORG_MEMORY_ID: MemoryId = MemoryId::new(16);
    pub const GIFTCARD_SPAWNORG_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(17);

    thread_local! { 
        // self info - immutable
        pub(crate) static CANISTER_ID: PublicKeyICP = PublicKeyICP(ic_cdk::api::id().to_text());
        
        // Convert regular variable to StableCell
        pub(crate) static VERSION: RefCell<StableCell<String, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(VERSION_MEMORY_ID)),
                "OfficeX.Beta.0.0.2".to_string()
            ).expect("Failed to initialize VERSION")
        );
        
        pub(crate) static OWNER_ID: RefCell<StableCell<UserID, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(OWNER_ID_MEMORY_ID)),
                UserID("Anonymous_Owner".to_string())
            ).expect("Failed to initialize OWNER_ID")
        );
        
        pub(crate) static URL_ENDPOINT: RefCell<StableCell<DriveRESTUrlEndpoint, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(URL_ENDPOINT_MEMORY_ID)),
                DriveRESTUrlEndpoint(format!("https://{}.icp0.io", CANISTER_ID.with(|id| id.0.clone())))
            ).expect("Failed to initialize URL_ENDPOINT")
        );
        
        // Convert HashMap to StableBTreeMap for deployments
        pub(crate) static DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID: RefCell<StableBTreeMap<GiftcardSpawnOrgID, FactorySpawnHistoryRecord, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID_MEMORY_ID))
            )
        );
        
        // Convert Vec to StableVec for historical records
        pub(crate) static HISTORICAL_GIFTCARDS_SPAWNORGS: RefCell<StableVec<GiftcardSpawnOrgID, Memory>> = RefCell::new(
            StableVec::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(HISTORICAL_GIFTCARDS_SPAWNORGS_MEMORY_ID))
            ).expect("Failed to initialize HISTORICAL_GIFTCARDS_SPAWNORGS")
        );
        
        // Convert HashMap to StableBTreeMap for drive mappings
        pub(crate) static DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE: RefCell<StableBTreeMap<DriveID, GiftcardSpawnOrgID, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DRIVE_TO_GIFTCARD_SPAWNORG_MEMORY_ID))
            )
        );
        
        // For HashMap<UserID, Vec<GiftcardSpawnOrgID>>, use a custom type (GiftcardSpawnOrgIDVec)
        pub(crate) static USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE: RefCell<StableBTreeMap<UserID, GiftcardSpawnOrgIDVec, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(USER_TO_GIFTCARDS_SPAWNORG_MEMORY_ID))
            )
        );
        
        // Convert HashMap to StableBTreeMap for giftcard storage
        pub(crate) static GIFTCARD_SPAWNORG_BY_ID: RefCell<StableBTreeMap<GiftcardSpawnOrgID, GiftcardSpawnOrg, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(GIFTCARD_SPAWNORG_BY_ID_MEMORY_ID))
            )
        );
    }


    pub fn initialize() {
        // Force thread_locals in this module to initialize
        VERSION.with(|_| {});
        OWNER_ID.with(|_| {});
        URL_ENDPOINT.with(|_| {});
        DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID.with(|_| {});
        HISTORICAL_GIFTCARDS_SPAWNORGS.with(|_| {});
        DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE.with(|_| {});
        USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE.with(|_| {});
        GIFTCARD_SPAWNORG_BY_ID.with(|_| {});
    }

    pub fn init_self_factory(
        owner_id: UserID,
    ) {
        debug_log!("Setting owner_id: {}", owner_id.0);
        OWNER_ID.with(|id| {
            // Use set() instead of direct assignment
            id.borrow_mut().set(owner_id.clone());
            debug_log!("Confirmed owner_id set to: {}", id.borrow().get().0);
        });
    
        // Handle the URL endpoint
        let endpoint = get_appropriate_url_endpoint();
        debug_log!("Setting URL endpoint to: {}", endpoint);
        URL_ENDPOINT.with(|url| {
            // Use set() instead of direct assignment
            url.borrow_mut().set(DriveRESTUrlEndpoint(endpoint));
            debug_log!("Confirmed URL endpoint set to: {}", url.borrow().get().0);
        });
    }
    
}