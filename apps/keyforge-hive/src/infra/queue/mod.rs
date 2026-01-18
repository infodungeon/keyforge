pub mod wrappers;
pub mod worker;

pub use worker::{PersistentJobQueue, DbEvent, BatchSink, QUEUE_MAX_RETRIES, QUEUE_RETRY_DELAY_MS};
pub use wrappers::{PersistedRecord, WalEntry, DeadLetterQueue};

// Backward compatibility alias until callers are updated
pub type WriteQueue = PersistentJobQueue;
