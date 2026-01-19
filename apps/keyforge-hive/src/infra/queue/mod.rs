pub mod wrappers;
pub mod worker;

pub use worker::{PersistentJobQueue, BatchSink};
pub use wrappers::{PersistedRecord, WalEntry, DeadLetterQueue};

// Backward compatibility alias until callers are updated
pub type WriteQueue = PersistentJobQueue;