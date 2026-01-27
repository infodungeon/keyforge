use keyforge_model::KeyCode;
use std::cell::RefCell;

thread_local! {
    pub static POS_MAP_SCRATCH: RefCell<Vec<keyforge_model::KeyIndex>> = RefCell::new(vec![keyforge_model::KeyIndex::SENTINEL; 65536]);
    pub static KEYS_SCRATCH: RefCell<Vec<KeyCode>> = RefCell::new(Vec::with_capacity(128));
}
