//! AudioContext fingerprinting defense.
//!
//! AudioContext fingerprinting uses oscillator nodes and analyser output.
//! We return deterministic noise to prevent fingerprinting while
//! maintaining audio functionality.

/// Audio defense configuration.
#[derive(Debug, Clone)]
pub struct AudioDefense {
    /// Seed for this identity
    seed: u64,
}

impl AudioDefense {
    /// Create a new audio defense.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate a deterministic audio fingerprint response.
    ///
    /// When a page tries to fingerprint via AudioContext, we return
    /// values from this function instead of real audio processing results.
    pub fn generate_fingerprint_data(&self, length: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..length)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                self.seed.hash(&mut hasher);
                i.hash(&mut hasher);
                let hash = hasher.finish();

                // Generate value between -1.0 and 1.0
                ((hash as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32
            })
            .collect()
    }

    /// Apply noise to frequency data from AnalyserNode.
    pub fn apply_frequency_noise(&self, data: &mut [f32]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        for (i, value) in data.iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            self.seed.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();

            // Add subtle noise
            let noise = ((hash as f64 / u64::MAX as f64) - 0.5) * 0.01;
            *value += noise as f32;
        }
    }

    /// Apply noise to time domain data.
    pub fn apply_time_domain_noise(&self, data: &mut [f32]) {
        // Same implementation as frequency for simplicity
        self.apply_frequency_noise(data);
    }

    /// Get spoofed audio context properties.
    pub fn get_audio_context_properties(&self) -> AudioContextProperties {
        AudioContextProperties {
            sample_rate: 48000.0,
            base_latency: 0.005333333333333333,
            output_latency: 0.016,
            max_channel_count: 2,
            state: "running".to_string(),
        }
    }

    /// Check if an audio method should have noise applied.
    pub fn should_apply_noise(method: &str) -> bool {
        matches!(
            method,
            "getFloatFrequencyData"
                | "getByteFrequencyData"
                | "getFloatTimeDomainData"
                | "getByteTimeDomainData"
                | "createOscillator"
                | "createDynamicsCompressor"
        )
    }
}

/// Spoofed AudioContext properties.
#[derive(Debug, Clone)]
pub struct AudioContextProperties {
    /// Sample rate
    pub sample_rate: f64,
    /// Base latency
    pub base_latency: f64,
    /// Output latency
    pub output_latency: f64,
    /// Max channel count
    pub max_channel_count: u32,
    /// Context state
    pub state: String,
}

/// Generate a deterministic DynamicsCompressor fingerprint.
///
/// The DynamicsCompressor is commonly used for fingerprinting.
/// We return consistent but non-unique values.
pub fn fake_dynamics_compressor_output(seed: u64) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Standard DynamicsCompressor output length
    let length = 128;

    (0..length)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            i.hash(&mut hasher);
            (hasher.finish() as f32 / u64::MAX as f32) * 0.1 - 0.05
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_fingerprint() {
        let defense = AudioDefense::new(42);

        let data1 = defense.generate_fingerprint_data(128);
        let data2 = defense.generate_fingerprint_data(128);

        assert_eq!(data1, data2);
    }

    #[test]
    fn test_different_seeds() {
        let defense1 = AudioDefense::new(42);
        let defense2 = AudioDefense::new(43);

        let data1 = defense1.generate_fingerprint_data(128);
        let data2 = defense2.generate_fingerprint_data(128);

        assert_ne!(data1, data2);
    }

    #[test]
    fn test_should_apply_noise() {
        assert!(AudioDefense::should_apply_noise("getFloatFrequencyData"));
        assert!(!AudioDefense::should_apply_noise("play"));
    }
}
