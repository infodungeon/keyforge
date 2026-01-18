use keyforge_model::KeyCode;
use std::cell::RefCell;

thread_local! {
    pub static POS_MAP_SCRATCH: RefCell<Vec<u16>> = RefCell::new(vec![0u16; 65536]);
    pub static KEYS_SCRATCH: RefCell<Vec<KeyCode>> = RefCell::new(Vec::with_capacity(128));
}
