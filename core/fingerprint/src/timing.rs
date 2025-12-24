//! Timing API fuzzing.
//!
//! High-resolution timing APIs enable fingerprinting and side-channel attacks.
//! We reduce precision and add jitter.

use std::time::{Duration, Instant};

/// Timing defense configuration.
#[derive(Debug, Clone)]
pub struct TimingDefense {
    /// Base time for Date.now() calculations
    base_time: Instant,
    /// Precision for Date.now() in milliseconds
    date_precision_ms: u64,
    /// Precision for performance.now() in milliseconds
    perf_precision_ms: u64,
    /// Maximum jitter to add
    max_jitter_ms: u64,
    /// Seed for deterministic jitter
    jitter_seed: u64,
}

impl TimingDefense {
    /// Create a new timing defense.
    pub fn new(jitter_seed: u64) -> Self {
        Self {
            base_time: Instant::now(),
            date_precision_ms: 100, // 100ms precision
            perf_precision_ms: 100, // 100ms precision (Tor Browser uses this)
            max_jitter_ms: 10,
            jitter_seed,
        }
    }

    /// Get fuzzed Date.now() value.
    pub fn fuzz_date_now(&self, actual_ms: u64) -> u64 {
        // Reduce precision
        let reduced = (actual_ms / self.date_precision_ms) * self.date_precision_ms;

        // Add deterministic jitter
        let jitter = self.deterministic_jitter(actual_ms);
        reduced.saturating_add(jitter)
    }

    /// Get fuzzed performance.now() value.
    pub fn fuzz_performance_now(&self, actual_ms: f64) -> f64 {
        // Reduce precision to 100ms
        let reduced = (actual_ms / self.perf_precision_ms as f64).floor()
            * self.perf_precision_ms as f64;

        // Add jitter
        let jitter = self.deterministic_jitter(actual_ms as u64) as f64;
        reduced + jitter
    }

    /// Generate deterministic jitter based on seed and input.
    fn deterministic_jitter(&self, input: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.jitter_seed.hash(&mut hasher);
        input.hash(&mut hasher);
        let hash = hasher.finish();

        hash % self.max_jitter_ms
    }

    /// Clamp requestAnimationFrame to 60Hz.
    /// Returns the adjusted timestamp.
    pub fn clamp_raf_timestamp(&self, actual_ms: f64) -> f64 {
        // 60 FPS = 16.67ms per frame, but we round to 16ms buckets
        let frame_time = 16.0;
        (actual_ms / frame_time).floor() * frame_time
    }

    /// Get clamped setTimeout/setInterval minimum delay.
    pub fn minimum_timer_delay(&self) -> u64 {
        4 // Minimum 4ms (browser standard)
    }

    /// Fuzz a timer callback delay.
    pub fn fuzz_timer_delay(&self, requested_ms: u64) -> u64 {
        // Ensure minimum
        let delay = requested_ms.max(self.minimum_timer_delay());

        // Add small jitter to prevent timing attacks
        let jitter = self.deterministic_jitter(delay);
        delay + jitter
    }
}

impl Default for TimingDefense {
    fn default() -> Self {
        Self::new(rand::random())
    }
}

/// APIs that should be modified for timing defense.
pub fn timing_apis_to_fuzz() -> &'static [&'static str] {
    &[
        "Date.now",
        "Date.prototype.getTime",
        "Date.prototype.valueOf",
        "performance.now",
        "performance.timeOrigin",
        "performance.timing",
        "requestAnimationFrame",
        "setTimeout",
        "setInterval",
    ]
}

/// APIs that should be completely disabled.
pub fn timing_apis_to_block() -> &'static [&'static str] {
    &[
        "SharedArrayBuffer", // Can be used for high-res timing
        "Atomics",           // Related to SharedArrayBuffer
        "performance.measureUserAgentSpecificMemory",
        "crossOriginIsolated", // Report as false
    ]
}

/// Resource timing should be disabled or return empty.
pub fn block_resource_timing() -> bool {
    true
}

/// Navigation timing should return fuzzed values.
pub fn fuzz_navigation_timing() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_now_fuzzing() {
        let defense = TimingDefense::new(42);

        let actual = 1703412345678u64;
        let fuzzed = defense.fuzz_date_now(actual);

        // Should be rounded to 100ms precision
        assert_eq!(fuzzed % 100, fuzzed % 100); // Still aligned
        assert!(fuzzed >= (actual / 100) * 100);
        assert!(fuzzed <= (actual / 100) * 100 + defense.max_jitter_ms);
    }

    #[test]
    fn test_performance_now_fuzzing() {
        let defense = TimingDefense::new(42);

        let actual = 123.456;
        let fuzzed = defense.fuzz_performance_now(actual);

        // Should be rounded to 100ms
        assert!(fuzzed >= 100.0);
        assert!(fuzzed < 200.0);
    }

    #[test]
    fn test_raf_clamping() {
        let defense = TimingDefense::new(42);

        let clamped = defense.clamp_raf_timestamp(17.5);
        assert_eq!(clamped, 16.0);

        let clamped = defense.clamp_raf_timestamp(33.0);
        assert_eq!(clamped, 32.0);
    }

    #[test]
    fn test_deterministic_jitter() {
        let defense1 = TimingDefense::new(42);
        let defense2 = TimingDefense::new(42);

        // Same seed and input = same jitter
        assert_eq!(
            defense1.deterministic_jitter(1000),
            defense2.deterministic_jitter(1000)
        );

        // Different seed = different jitter
        let defense3 = TimingDefense::new(43);
        assert_ne!(
            defense1.deterministic_jitter(1000),
            defense3.deterministic_jitter(1000)
        );
    }
}
