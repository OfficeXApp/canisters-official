use std::{cell::RefCell, collections::HashMap};
use crate::rest::{apikeys::types::ApiKeyItem, templates::types::TemplateItem};

thread_local! {
    // Template State
    pub static NEXT_TEMPLATE_ID: RefCell<u32> = RefCell::new(0);
    pub static TEMPLATE_ITEMS: RefCell<HashMap<u32, TemplateItem>> = RefCell::new(HashMap::new());
    // ApiKeys State
    pub static NEXT_APIKEY_ID: RefCell<u32> = RefCell::new(0);
    pub static APIKEY_ITEMS: RefCell<HashMap<u32, ApiKeyItem>> = RefCell::new(HashMap::new());
}