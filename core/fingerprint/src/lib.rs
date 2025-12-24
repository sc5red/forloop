//! Fingerprint Defense Module for forloop browser.
//!
//! This crate provides comprehensive fingerprinting defenses including:
//! - Canvas fingerprint spoofing
//! - WebGL fingerprint spoofing
//! - AudioContext fingerprint spoofing
//! - Font enumeration blocking
//! - Screen/window size normalization
//! - Hardware property spoofing
//! - Timing API fuzzing
//!
//! All defenses produce deterministic outputs from a large anonymity set.

#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod canvas;
pub mod webgl;
pub mod audio;
pub mod fonts;
pub mod screen;
pub mod hardware;
pub mod timing;
pub mod navigator;

use std::sync::Arc;

/// Synthetic identity for fingerprint consistency within a request.
///
/// All fingerprinting APIs return values derived from this identity.
/// The identity changes per request, making correlation impossible.
#[derive(Debug, Clone)]
pub struct SyntheticIdentity {
    /// Seed for deterministic random generation
    seed: [u8; 32],
    /// Canvas noise seed
    pub canvas_seed: u64,
    /// WebGL noise seed
    pub webgl_seed: u64,
    /// Audio noise seed
    pub audio_seed: u64,
    /// Timezone offset (minutes from UTC)
    pub timezone_offset: i32,
    /// Platform string
    pub platform: String,
    /// Screen bucket
    pub screen_bucket: screen::ScreenBucket,
    /// Hardware profile
    pub hardware: hardware::HardwareProfile,
}

impl SyntheticIdentity {
    /// Generate a new random synthetic identity.
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut seed = [0u8; 32];
        rng.fill(&mut seed);

        Self::from_seed(seed)
    }

    /// Create a synthetic identity from a seed.
    /// This allows reproducible identities for testing.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha20Rng;

        let mut rng = ChaCha20Rng::from_seed(seed);

        // Select from anonymity sets
        let timezones = [-480, -420, -360, -300, -240, 0, 60, 120, 180];
        let platforms = ["Win32", "Linux x86_64", "MacIntel"];

        Self {
            seed,
            canvas_seed: rng.gen(),
            webgl_seed: rng.gen(),
            audio_seed: rng.gen(),
            timezone_offset: *timezones.choose(&mut rng).unwrap_or(&0),
            platform: platforms.choose(&mut rng).unwrap_or(&"Linux x86_64").to_string(),
            screen_bucket: screen::ScreenBucket::random(&mut rng),
            hardware: hardware::HardwareProfile::random(&mut rng),
        }
    }

    /// Get the identity seed.
    pub fn seed(&self) -> &[u8; 32] {
        &self.seed
    }
}

/// Trait for types that can be randomized within a given set.
trait RandomChoice: Sized {
    /// Choose a random value from the defined set.
    fn random<R: rand::Rng>(rng: &mut R) -> Self;
}

/// Global fingerprint defense controller.
pub struct FingerprintDefense {
    identity: Arc<SyntheticIdentity>,
}

impl FingerprintDefense {
    /// Create a new fingerprint defense with a random identity.
    pub fn new() -> Self {
        Self {
            identity: Arc::new(SyntheticIdentity::generate()),
        }
    }

    /// Create with a specific identity.
    pub fn with_identity(identity: SyntheticIdentity) -> Self {
        Self {
            identity: Arc::new(identity),
        }
    }

    /// Get the current synthetic identity.
    pub fn identity(&self) -> &SyntheticIdentity {
        &self.identity
    }

    /// Rotate to a new identity (call between requests).
    pub fn rotate(&mut self) {
        self.identity = Arc::new(SyntheticIdentity::generate());
    }
}

impl Default for FingerprintDefense {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let id1 = SyntheticIdentity::generate();
        let id2 = SyntheticIdentity::generate();

        // Different identities should have different seeds
        assert_ne!(id1.seed, id2.seed);
    }

    #[test]
    fn test_identity_reproducibility() {
        let seed = [42u8; 32];
        let id1 = SyntheticIdentity::from_seed(seed);
        let id2 = SyntheticIdentity::from_seed(seed);

        // Same seed should produce same values
        assert_eq!(id1.canvas_seed, id2.canvas_seed);
        assert_eq!(id1.timezone_offset, id2.timezone_offset);
        assert_eq!(id1.platform, id2.platform);
    }

    #[test]
    fn test_defense_rotation() {
        let mut defense = FingerprintDefense::new();
        let seed1 = *defense.identity().seed();

        defense.rotate();
        let seed2 = *defense.identity().seed();

        assert_ne!(seed1, seed2);
    }
}
