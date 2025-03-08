// src/core/state/drives/state.rs

pub mod state {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::api::helpers::get_appropriate_url_endpoint;
    use crate::core::state::vouchers::types::DriveID;
    use crate::core::state::vouchers::types::DriveRESTUrlEndpoint;
    use crate::core::state::vouchers::types::FactorySpawnHistoryRecord;
    use crate::core::state::vouchers::types::VoucherID;
    use crate::core::state::vouchers::types::Voucher;

    use crate::core::types::{UserID,PublicKeyICP};
    use crate::debug_log;

    thread_local! { 
        // self info - immutable
        pub(crate) static CANISTER_ID: PublicKeyICP = PublicKeyICP(ic_cdk::api::id().to_text());
        pub(crate) static GLOBAL_UUID_NONCE: Cell<u64> = Cell::new(0);
        pub(crate) static VERSION: RefCell<String> = RefCell::new("OfficeX.Beta.0.0.1".to_string());
        pub(crate) static OWNER_ID: RefCell<UserID> = RefCell::new(UserID("Anonymous_Owner".to_string()));
        pub(crate) static URL_ENDPOINT: RefCell<DriveRESTUrlEndpoint> = RefCell::new(DriveRESTUrlEndpoint(format!("https://{}.icp0.io", CANISTER_ID.with(|id| id.0.clone()))));
        
        // Voucher and deployment tracking
        pub(crate) static DEPLOYMENTS_BY_VOUCHER_ID: RefCell<HashMap<VoucherID, FactorySpawnHistoryRecord>> = RefCell::new(HashMap::new());
        pub(crate) static HISTORICAL_VOUCHERS: RefCell<Vec<VoucherID>> = RefCell::new(Vec::new());
        pub(crate) static DRIVE_TO_VOUCHER_HASHTABLE: RefCell<HashMap<DriveID, VoucherID>> = RefCell::new(HashMap::new());
        pub(crate) static USER_TO_VOUCHERS_HASHTABLE: RefCell<HashMap<UserID, Vec<VoucherID>>> = RefCell::new(HashMap::new());
        pub(crate) static VOUCHER_BY_ID: RefCell<HashMap<VoucherID, Voucher>> = RefCell::new(HashMap::new());
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