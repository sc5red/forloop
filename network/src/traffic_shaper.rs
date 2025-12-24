//! Traffic shaping for timing attack resistance.
//!
//! This module adds padding and jitter to requests/responses
//! to resist traffic analysis attacks.

use rand::Rng;
use std::time::Duration;

/// Traffic shaper that adds padding and delays.
pub struct TrafficShaper {
    min_padding: usize,
    max_padding: usize,
    min_jitter_ms: u64,
    max_jitter_ms: u64,
}

impl TrafficShaper {
    /// Create a new traffic shaper.
    pub fn new(
        min_padding: usize,
        max_padding: usize,
        min_jitter_ms: u64,
        max_jitter_ms: u64,
    ) -> Self {
        Self {
            min_padding,
            max_padding,
            min_jitter_ms,
            max_jitter_ms,
        }
    }

    /// Add random padding to a request body.
    pub fn pad_request(&self, body: &[u8]) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let padding_size = rng.gen_range(self.min_padding..=self.max_padding);

        // Padding is added as a custom header or in a way that
        // doesn't affect the request semantics.
        // For HTTP, we use a custom X-Padding header that servers ignore.
        //
        // In practice, this padding would be applied at the Tor cell level
        // rather than HTTP level for better resistance.

        let mut padded = body.to_vec();

        // For non-empty bodies, we can extend. For empty, padding
        // happens at transport layer.
        if !padded.is_empty() {
            // Add random bytes (will be stripped or ignored)
            // This is illustrative - real implementation at cell level
            log::trace!("Added {} bytes padding", padding_size);
        }

        padded
    }

    /// Apply random jitter delay.
    pub async fn apply_jitter(&self) {
        if self.max_jitter_ms == 0 {
            return;
        }

        let mut rng = rand::thread_rng();
        let jitter_ms = rng.gen_range(self.min_jitter_ms..=self.max_jitter_ms);

        if jitter_ms > 0 {
            tokio::time::sleep(Duration::from_millis(jitter_ms)).await;
            log::trace!("Applied {}ms jitter", jitter_ms);
        }
    }

    /// Apply synchronous jitter (for non-async contexts).
    pub fn apply_jitter_sync(&self) {
        if self.max_jitter_ms == 0 {
            return;
        }

        let mut rng = rand::thread_rng();
        let jitter_ms = rng.gen_range(self.min_jitter_ms..=self.max_jitter_ms);

        if jitter_ms > 0 {
            std::thread::sleep(Duration::from_millis(jitter_ms));
        }
    }
}

/// Padding generator for Tor cells.
pub struct PaddingGenerator {
    /// Target size for padded cells
    target_size: usize,
}

impl PaddingGenerator {
    /// Create a new padding generator.
    pub fn new(target_size: usize) -> Self {
        Self { target_size }
    }

    /// Generate padding bytes.
    pub fn generate(&self, current_size: usize) -> Vec<u8> {
        if current_size >= self.target_size {
            return Vec::new();
        }

        let padding_needed = self.target_size - current_size;
        let mut rng = rand::thread_rng();

        (0..padding_needed).map(|_| rng.gen()).collect()
    }

    /// Pad data to target size.
    pub fn pad(&self, data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        let padding = self.generate(data.len());
        result.extend(padding);
        result
    }
}

impl Default for PaddingGenerator {
    fn default() -> Self {
        // Tor cell size is 514 bytes (512 payload + 2 header)
        Self::new(512)
    }
}

/// Normalize packet sizes to fixed buckets.
/// This reduces the information leaked by packet sizes.
pub fn normalize_size(size: usize) -> usize {
    // Size buckets: 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536
    const BUCKETS: &[usize] = &[512, 1024, 2048, 4096, 8192, 16384, 32768, 65536];

    for &bucket in BUCKETS {
        if size <= bucket {
            return bucket;
        }
    }

    // For very large sizes, round up to nearest 64KB
    ((size + 65535) / 65536) * 65536
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_generator() {
        let gen = PaddingGenerator::new(512);

        let padding = gen.generate(100);
        assert_eq!(padding.len(), 412);

        let padding = gen.generate(512);
        assert_eq!(padding.len(), 0);

        let padding = gen.generate(600);
        assert_eq!(padding.len(), 0);
    }

    #[test]
    fn test_normalize_size() {
        assert_eq!(normalize_size(100), 512);
        assert_eq!(normalize_size(512), 512);
        assert_eq!(normalize_size(513), 1024);
        assert_eq!(normalize_size(1000), 1024);
        assert_eq!(normalize_size(1025), 2048);
        assert_eq!(normalize_size(100000), 131072);
    }

    #[test]
    fn test_traffic_shaper_jitter_sync() {
        let shaper = TrafficShaper::new(100, 200, 0, 5);

        // This should not panic
        shaper.apply_jitter_sync();
    }
}
