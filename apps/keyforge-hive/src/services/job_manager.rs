// apps/keyforge-hive/src/services/job_manager.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::infra::queue::WriteQueue;
use crate::infra::repositories::JobRepository;
use std::sync::Arc;
use tokio::sync::{Notify, Semaphore};

#[derive(Clone, Debug)]
pub struct JobManager {
    pub repo: JobRepository,
    pub queue: Arc<WriteQueue>,
    pub signal: Arc<Notify>,
    pub semaphore: Arc<Semaphore>,
    pub active_count: Arc<std::sync::atomic::AtomicUsize>,
    pub completed_count: Arc<std::sync::atomic::AtomicUsize>,
}

use crate::config::DEFAULT_QUEUE_CHANNEL_CAPACITY;

impl JobManager {
    pub fn new(
        repo: JobRepository,
        queue: Arc<WriteQueue>,
    ) -> Self {
        Self {
            repo,
            queue,
            signal: Arc::new(Notify::new()),
            semaphore: Arc::new(Semaphore::new(DEFAULT_QUEUE_CHANNEL_CAPACITY)),
            active_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            completed_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}
