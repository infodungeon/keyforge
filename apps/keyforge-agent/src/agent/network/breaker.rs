use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct CircuitBreaker {
    failures: u32,
    threshold: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, cooldown_secs: u64) -> Self {
        Self { 
            failures: 0, 
            threshold, 
            last_failure: None, 
            cooldown: Duration::from_secs(cooldown_secs) 
        }
    }

    pub fn can_attempt(&self) -> bool {
        if self.failures < self.threshold { 
            return true; 
        }
        if let Some(last) = self.last_failure { 
            if last.elapsed() > self.cooldown { 
                return true; 
            } 
        }
        false
    }

    pub fn record_failure(&mut self) { 
        self.failures += 1; 
        self.last_failure = Some(Instant::now()); 
    }

    pub fn record_success(&mut self) { 
        self.failures = 0; 
        self.last_failure = None; 
    }
}
