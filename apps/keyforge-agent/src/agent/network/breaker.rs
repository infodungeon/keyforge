use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    failures: u32,
    threshold: u32,
    last_failure: Option<Instant>,
    cooldown: Duration,
    state: State,
}

impl CircuitBreaker {
    #[must_use]
    pub fn new(threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            failures: 0,
            threshold,
            last_failure: None,
            cooldown: Duration::from_secs(cooldown_secs),
            state: State::Closed,
        }
    }

    #[must_use]
    pub fn can_attempt(&mut self) -> bool {
        match self.state {
            State::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() > self.cooldown {
                        self.state = State::HalfOpen;
                        return true;
                    }
                }
                false
            }
            State::Closed | State::HalfOpen => {
                // In Closed or HalfOpen, we allow an attempt.
                true
            }
        }
    }

    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(Instant::now());
        if self.failures >= self.threshold || self.state == State::HalfOpen {
            self.state = State::Open;
        }
    }

    pub fn record_success(&mut self) {
        self.failures = 0;
        self.last_failure = None;
        self.state = State::Closed;
    }
}

#[keyforge_testing_macros::kf_test]
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
            "Breaker should allow attempt after cooldown (Half-Open)"
        );
        assert_eq!(cb.state, State::HalfOpen);

        cb.record_success();
        assert!(cb.can_attempt());
        assert_eq!(cb.state, State::Closed);
    }
}
