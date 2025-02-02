// src/core/state/templates/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::templates::types::{TemplateID, TemplateItem};
    
    thread_local! {
        pub static TEMPLATE_ITEMS: RefCell<HashMap<TemplateID, TemplateItem>> = RefCell::new(HashMap::new());
    }

}


