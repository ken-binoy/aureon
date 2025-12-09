//! Error Recovery and Resilience for Production
//!
//! Provides comprehensive error handling, retry logic, circuit breakers,
//! and graceful degradation for production-grade operations.

use std::time::{Duration, SystemTime};
use std::collections::VecDeque;

/// Custom error type for recoverable operations
#[derive(Debug, Clone)]
pub enum RecoveryError {
    /// Temporary network error
    TemporaryError(String),
    /// Permanent error that won't recover
    PermanentError(String),
    /// Timeout error
    TimeoutError(String),
    /// Circuit breaker is open
    CircuitBreakerOpen,
    /// Rate limit exceeded
    RateLimited,
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::TemporaryError(msg) => write!(f, "Temporary error: {}", msg),
            RecoveryError::PermanentError(msg) => write!(f, "Permanent error: {}", msg),
            RecoveryError::TimeoutError(msg) => write!(f, "Timeout: {}", msg),
            RecoveryError::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            RecoveryError::RateLimited => write!(f, "Rate limited"),
        }
    }
}

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration
    pub max_backoff_ms: u64,
    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate backoff duration for attempt number
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let backoff_ms = (self.initial_backoff_ms as f64
            * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let capped = backoff_ms.min(self.max_backoff_ms);
        Duration::from_millis(capped)
    }
}

/// Circuit breaker state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests fail immediately
    Open,
    /// Circuit is half-open, test requests allowed
    HalfOpen,
}

/// Circuit breaker for preventing cascade failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state of the circuit
    state: CircuitState,
    /// Number of consecutive failures
    failure_count: u32,
    /// Failure threshold before opening
    failure_threshold: u32,
    /// Number of successes needed to close from half-open
    success_threshold: u32,
    /// Current successes in half-open state
    success_count: u32,
    /// Timestamp of last state change
    last_state_change: SystemTime,
    /// Duration to wait before half-open
    timeout_duration: Duration,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        CircuitBreaker::new(5, 2, Duration::from_secs(30))
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            success_threshold,
            success_count: 0,
            last_state_change: SystemTime::now(),
            timeout_duration: timeout,
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.last_state_change = SystemTime::now();
                }
            }
            CircuitState::Open => {
                // Ignore successes when open
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_state_change = SystemTime::now();
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.failure_count = 1;
                self.success_count = 0;
                self.last_state_change = SystemTime::now();
            }
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Ok(elapsed) = self.last_state_change.elapsed() {
                    if elapsed >= self.timeout_duration {
                        self.state = CircuitState::HalfOpen;
                        self.failure_count = 0;
                        self.success_count = 0;
                        self.last_state_change = SystemTime::now();
                    }
                }
            }
        }
    }

    /// Check if request is allowed
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Ok(elapsed) = self.last_state_change.elapsed() {
                    if elapsed >= self.timeout_duration {
                        self.state = CircuitState::HalfOpen;
                        self.failure_count = 0;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// Rate limiter using token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum tokens (capacity)
    capacity: u32,
    /// Current tokens available
    tokens: u32,
    /// Tokens per second refill rate
    refill_rate: u32,
    /// Last refill timestamp
    last_refill: SystemTime,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        RateLimiter {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: SystemTime::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        if let Ok(elapsed) = self.last_refill.elapsed() {
            let seconds = elapsed.as_secs() as u32;
            if seconds > 0 {
                self.tokens = (self.tokens + seconds * self.refill_rate).min(self.capacity);
                self.last_refill = SystemTime::now();
            }
        }
    }

    /// Try to acquire a token (return true if allowed)
    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get available tokens
    pub fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens
    }
}

/// Error recovery context for tracking operation state
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// Current retry attempt (0-indexed)
    pub attempt: u32,
    /// Retry configuration
    pub config: RetryConfig,
    /// Errors from previous attempts
    pub error_history: VecDeque<RecoveryError>,
}

impl RecoveryContext {
    /// Create a new recovery context
    pub fn new(config: RetryConfig) -> Self {
        RecoveryContext {
            attempt: 0,
            config,
            error_history: VecDeque::new(),
        }
    }

    /// Check if more retries are allowed
    pub fn can_retry(&self) -> bool {
        self.attempt < self.config.max_retries
    }

    /// Record an error and advance attempt
    pub fn record_error(&mut self, error: RecoveryError) -> bool {
        self.error_history.push_back(error);
        if self.error_history.len() > 10 {
            self.error_history.pop_front();
        }
        self.attempt += 1;
        self.can_retry()
    }

    /// Get backoff duration for next attempt
    pub fn next_backoff(&self) -> Duration {
        self.config.calculate_backoff(self.attempt)
    }

    /// Get last error if any
    pub fn last_error(&self) -> Option<&RecoveryError> {
        self.error_history.back()
    }
}

/// Graceful degradation fallback strategy
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Return cached value if available
    UseCache,
    /// Return default/empty value
    UseDefault,
    /// Return partial result with what's available
    PartialResult,
    /// Fail with error
    FailFast,
}

/// Health check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but operational
    Degraded,
    /// System is unhealthy
    Unhealthy,
}

/// Health checker for component monitoring
#[derive(Debug, Clone)]
pub struct HealthChecker {
    /// Current health status
    pub status: HealthStatus,
    /// Last check timestamp
    pub last_check: SystemTime,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Threshold for marking unhealthy
    pub failure_threshold: u32,
}

impl Default for HealthChecker {
    fn default() -> Self {
        HealthChecker {
            status: HealthStatus::Healthy,
            last_check: SystemTime::now(),
            failure_count: 0,
            failure_threshold: 5,
        }
    }
}

impl HealthChecker {
    /// Record a successful health check
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.status = HealthStatus::Healthy;
        self.last_check = SystemTime::now();
    }

    /// Record a failed health check
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_check = SystemTime::now();

        if self.failure_count >= self.failure_threshold {
            self.status = HealthStatus::Unhealthy;
        } else if self.failure_count > 0 {
            self.status = HealthStatus::Degraded;
        }
    }

    /// Reset health checker
    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.status = HealthStatus::Healthy;
        self.last_check = SystemTime::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 100);
    }

    #[test]
    fn test_retry_config_calculate_backoff() {
        let config = RetryConfig::default();
        let backoff_0 = config.calculate_backoff(0);
        let backoff_1 = config.calculate_backoff(1);
        let backoff_2 = config.calculate_backoff(2);

        assert!(backoff_1 > backoff_0);
        assert!(backoff_2 > backoff_1);
    }

    #[test]
    fn test_retry_config_backoff_capped() {
        let config = RetryConfig {
            initial_backoff_ms: 100,
            max_backoff_ms: 1000,
            ..RetryConfig::default()
        };

        let backoff = config.calculate_backoff(10);
        assert!(backoff.as_millis() <= 1000);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_open_on_failures() {
        let mut cb = CircuitBreaker::new(3, 2, Duration::from_secs(30));
        
        for _ in 0..3 {
            cb.record_failure();
        }
        
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_allow_request() {
        let mut cb = CircuitBreaker::new(3, 2, Duration::from_secs(30));
        
        assert!(cb.allow_request());
        
        for _ in 0..3 {
            cb.record_failure();
        }
        
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let mut cb = CircuitBreaker::new(2, 2, Duration::from_millis(100));
        
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        
        std::thread::sleep(Duration::from_millis(150));
        
        // After timeout, next call should check if we can move to half-open
        if cb.allow_request() {
            cb.record_failure();  // Fail in half-open
            // After failure in half-open, should be back to open
            assert_eq!(cb.state(), CircuitState::Open);
        }
    }

    #[test]
    fn test_circuit_breaker_close_from_half_open() {
        let mut cb = CircuitBreaker::new(2, 2, Duration::from_millis(100));
        
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        
        std::thread::sleep(Duration::from_millis(150));
        cb.allow_request();  // Transition to half-open
        
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10, 2);
        assert_eq!(limiter.capacity, 10);
        assert_eq!(limiter.tokens, 10);
    }

    #[test]
    fn test_rate_limiter_acquire() {
        let mut limiter = RateLimiter::new(3, 1);
        
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_rate_limiter_refill() {
        let mut limiter = RateLimiter::new(2, 1);
        
        // Use all tokens
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(!limiter.try_acquire());
        
        // Wait for refill
        std::thread::sleep(Duration::from_secs(1));
        
        // Should have at least one token refilled
        assert!(limiter.try_acquire());
    }

    #[test]
    fn test_recovery_context_creation() {
        let ctx = RecoveryContext::new(RetryConfig::default());
        assert_eq!(ctx.attempt, 0);
        assert!(ctx.can_retry());
    }

    #[test]
    fn test_recovery_context_record_error() {
        let mut ctx = RecoveryContext::new(RetryConfig::default());
        
        let can_retry = ctx.record_error(RecoveryError::TemporaryError("test".to_string()));
        assert!(can_retry);
        assert_eq!(ctx.attempt, 1);
        assert_eq!(ctx.error_history.len(), 1);
    }

    #[test]
    fn test_recovery_context_max_retries() {
        let config = RetryConfig {
            max_retries: 2,
            ..RetryConfig::default()
        };
        let mut ctx = RecoveryContext::new(config);
        
        ctx.record_error(RecoveryError::TemporaryError("test".to_string()));
        let can_retry = ctx.record_error(RecoveryError::TemporaryError("test".to_string()));
        
        assert!(!can_retry);
    }

    #[test]
    fn test_health_checker_default() {
        let hc = HealthChecker::default();
        assert_eq!(hc.status, HealthStatus::Healthy);
        assert_eq!(hc.failure_count, 0);
    }

    #[test]
    fn test_health_checker_record_failure() {
        let mut hc = HealthChecker::default();
        
        hc.record_failure();
        assert_eq!(hc.status, HealthStatus::Degraded);
        
        for _ in 0..4 {
            hc.record_failure();
        }
        assert_eq!(hc.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_checker_reset() {
        let mut hc = HealthChecker::default();
        
        for _ in 0..5 {
            hc.record_failure();
        }
        assert_eq!(hc.status, HealthStatus::Unhealthy);
        
        hc.reset();
        assert_eq!(hc.status, HealthStatus::Healthy);
        assert_eq!(hc.failure_count, 0);
    }

    #[test]
    fn test_health_checker_record_success() {
        let mut hc = HealthChecker::default();
        
        hc.record_failure();
        hc.record_failure();
        assert_eq!(hc.status, HealthStatus::Degraded);
        
        hc.record_success();
        assert_eq!(hc.status, HealthStatus::Healthy);
        assert_eq!(hc.failure_count, 0);
    }

    #[test]
    fn test_recovery_error_display() {
        let err = RecoveryError::TemporaryError("test error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_circuit_breaker_record_success_when_closed() {
        let mut cb = CircuitBreaker::default();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count, 0);
    }

    #[test]
    fn test_rate_limiter_available_tokens() {
        let mut limiter = RateLimiter::new(5, 1);
        assert_eq!(limiter.available_tokens(), 5);
        
        limiter.try_acquire();
        limiter.try_acquire();
        assert_eq!(limiter.available_tokens(), 3);
    }
}
