// src/core/state/drives/state.rs

pub mod state {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::api::helpers::get_appropriate_url_endpoint;
    use crate::core::state::giftcards_spawnorg::types::DriveID;
    use crate::core::state::giftcards_spawnorg::types::DriveRESTUrlEndpoint;
    use crate::core::state::giftcards_spawnorg::types::FactorySpawnHistoryRecord;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrgID;
    use crate::core::state::giftcards_spawnorg::types::GiftcardSpawnOrg;

    use crate::core::types::{UserID,PublicKeyICP};
    use crate::debug_log;

    thread_local! { 
        // self info - immutable
        pub(crate) static CANISTER_ID: PublicKeyICP = PublicKeyICP(ic_cdk::api::id().to_text());
        pub(crate) static VERSION: RefCell<String> = RefCell::new("OfficeX.Beta.0.0.1".to_string());
        pub(crate) static OWNER_ID: RefCell<UserID> = RefCell::new(UserID("Anonymous_Owner".to_string()));
        pub(crate) static URL_ENDPOINT: RefCell<DriveRESTUrlEndpoint> = RefCell::new(DriveRESTUrlEndpoint(format!("https://{}.icp0.io", CANISTER_ID.with(|id| id.0.clone()))));
        
        // GiftcardSpawnOrg and deployment tracking
        pub(crate) static DEPLOYMENTS_BY_GIFTCARD_SPAWNORG_ID: RefCell<HashMap<GiftcardSpawnOrgID, FactorySpawnHistoryRecord>> = RefCell::new(HashMap::new());
        pub(crate) static HISTORICAL_GIFTCARDS_SPAWNORGS: RefCell<Vec<GiftcardSpawnOrgID>> = RefCell::new(Vec::new());
        pub(crate) static DRIVE_TO_GIFTCARD_SPAWNORG_HASHTABLE: RefCell<HashMap<DriveID, GiftcardSpawnOrgID>> = RefCell::new(HashMap::new());
        pub(crate) static USER_TO_GIFTCARDS_SPAWNORG_HASHTABLE: RefCell<HashMap<UserID, Vec<GiftcardSpawnOrgID>>> = RefCell::new(HashMap::new());
        pub(crate) static GIFTCARD_SPAWNORG_BY_ID: RefCell<HashMap<GiftcardSpawnOrgID, GiftcardSpawnOrg>> = RefCell::new(HashMap::new());
    }

    pub fn init_self_factory(
        owner_id: UserID,
    ) {
        debug_log!("Setting owner_id: {}", owner_id.0);
        OWNER_ID.with(|id| {
            *id.borrow_mut() = owner_id.clone();
            debug_log!("Confirmed owner_id set to: {}", id.borrow().0);
        });

        // Handle the URL endpoint
        let endpoint = get_appropriate_url_endpoint();
        debug_log!("Setting URL endpoint to: {}", endpoint);
        URL_ENDPOINT.with(|url| {
            *url.borrow_mut() = DriveRESTUrlEndpoint(endpoint);
            debug_log!("Confirmed URL endpoint set to: {}", url.borrow().0);
        });
        
    }
    
}