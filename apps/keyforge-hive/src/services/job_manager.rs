use crate::infra::queue::WriteQueue;
use crate::infra::repositories::JobRepository;
use std::sync::Arc;
use tokio::sync::{Notify, Semaphore};

#[derive(Clone)]
pub struct JobManager {
    pub repo: JobRepository,
    pub queue: Arc<WriteQueue>,
    pub signal: Arc<Notify>,
    pub semaphore: Arc<Semaphore>,
    pub active_count: Arc<std::sync::atomic::AtomicUsize>,
    pub completed_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl JobManager {
    pub fn new(
        repo: JobRepository,
        queue: Arc<WriteQueue>,
    ) -> Self {
        Self {
            repo,
            queue,
            signal: Arc::new(Notify::new()),
            semaphore: Arc::new(Semaphore::new(1000)),
            active_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            completed_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}
