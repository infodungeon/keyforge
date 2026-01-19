use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct CircuitBreaker {
    failures: u32,
    threshold: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
}

impl CircuitBreaker {
    #[must_use]
    pub fn new(threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            failures: 0,
            threshold,
            last_failure: None,
            cooldown: Duration::from_secs(cooldown_secs),
        }
    }

    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_tripping() {
        let mut cb = CircuitBreaker::new(2, 1); // 2 failures, 1s cooldown

        assert!(cb.can_attempt());

        cb.record_failure();
        assert!(cb.can_attempt());

        cb.record_failure();
        assert!(!cb.can_attempt(), "Breaker should be tripped");

        // Wait for cooldown
        std::thread::sleep(Duration::from_millis(1100));
        assert!(
            cb.can_attempt(),
            "Breaker should allow attempt after cooldown"
        );

        cb.record_success();
        assert!(cb.can_attempt());
    }
}
