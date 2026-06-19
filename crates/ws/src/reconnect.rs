use std::time::Duration;

/// Configuration for WebSocket reconnection strategy.
///
/// The default configuration is:
/// - `max_retries`: `None` (infinite retries)
/// - `initial_delay`: 1 second
/// - `max_delay`: 60 seconds
/// - `backoff_factor`: 2.0
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// The maximum number of reconnection attempts before giving up.
    /// Set to `None` for infinite retries.
    pub max_retries: Option<usize>,
    /// The initial delay before the first reconnection attempt.
    pub initial_delay: Duration,
    /// The maximum delay between reconnection attempts.
    pub max_delay: Duration,
    /// The multiplier factor applied to the delay on each subsequent attempt.
    pub backoff_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_retries: None,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2.0,
        }
    }
}

impl ReconnectConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_retries(mut self, max_retries: Option<usize>) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn initial_delay(mut self, initial_delay: Duration) -> Self {
        self.initial_delay = initial_delay;
        self
    }

    pub fn max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    pub fn backoff_factor(mut self, backoff_factor: f64) -> Self {
        self.backoff_factor = backoff_factor;
        self
    }

    /// Calculates the delay for the given attempt index (1-based).
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let factor = self.backoff_factor.powi((attempt - 1) as i32);
        let secs = self.initial_delay.as_secs_f64() * factor;
        let delay = Duration::from_secs_f64(secs);
        std::cmp::min(delay, self.max_delay)
    }
}
