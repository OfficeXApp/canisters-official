// src/core/state/templates/state.rs
pub mod state {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::core::state::templates::types::TemplateItem;
    

    thread_local! {
        pub static NEXT_TEMPLATE_ID: RefCell<u32> = RefCell::new(0);
        pub static TEMPLATE_ITEMS: RefCell<HashMap<u32, TemplateItem>> = RefCell::new(HashMap::new());
    }

    pub struct TemplateState {
        pub next_id: u32,
        pub items: HashMap<u32, TemplateItem>,
    }

    impl Default for TemplateState {
        fn default() -> Self {
            Self {
                next_id: 0,
                items: HashMap::new(),
            }
        }
    }
}


