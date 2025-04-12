// src/core/state/drives/state.rs

pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use crate::core::state::giftcards_refuel::types::FactoryRefuelHistoryRecord;
    use crate::core::state::giftcards_spawnorg::types::DriveID;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuelID;
    use crate::core::state::giftcards_refuel::types::GiftcardRefuel;

    use crate::core::types::{UserID};
    

    thread_local! { 
        // GiftcardRefuel and deployment tracking
        pub(crate) static DEPLOYMENTS_BY_GIFTCARD_REFUEL_ID: RefCell<HashMap<GiftcardRefuelID, FactoryRefuelHistoryRecord>> = RefCell::new(HashMap::new());
        pub(crate) static HISTORICAL_GIFTCARDS_REFUELS: RefCell<Vec<GiftcardRefuelID>> = RefCell::new(Vec::new());
        pub(crate) static DRIVE_TO_GIFTCARD_REFUEL_HASHTABLE: RefCell<HashMap<DriveID, GiftcardRefuelID>> = RefCell::new(HashMap::new());
        pub(crate) static USER_TO_GIFTCARDS_REFUEL_HASHTABLE: RefCell<HashMap<UserID, Vec<GiftcardRefuelID>>> = RefCell::new(HashMap::new());
        pub(crate) static GIFTCARD_REFUEL_BY_ID: RefCell<HashMap<GiftcardRefuelID, GiftcardRefuel>> = RefCell::new(HashMap::new());
    }

    
}