//! Canvas fingerprinting defense.
//!
//! Canvas fingerprinting works by drawing content and reading back pixel data.
//! We inject deterministic noise based on the synthetic identity.

use super::RandomChoice;

/// Canvas defense configuration.
#[derive(Debug, Clone)]
pub struct CanvasDefense {
    /// Noise seed for this identity
    seed: u64,
    /// Noise intensity (0.0 - 1.0)
    intensity: f64,
}

impl CanvasDefense {
    /// Create a new canvas defense.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            intensity: 0.01, // 1% noise
        }
    }

    /// Apply noise to canvas image data.
    ///
    /// This modifies pixel values deterministically based on position and seed.
    /// The same content + position + seed always produces the same output,
    /// making it consistent within a page but different across identities.
    pub fn apply_noise(&self, data: &mut [u8], width: u32, height: u32) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 >= data.len() {
                    continue;
                }

                // Generate deterministic noise for this pixel
                let mut hasher = DefaultHasher::new();
                self.seed.hash(&mut hasher);
                x.hash(&mut hasher);
                y.hash(&mut hasher);
                let hash = hasher.finish();

                // Apply subtle noise to RGB (not alpha)
                for i in 0..3 {
                    let noise = ((hash >> (i * 8)) & 0xFF) as i16;
                    let delta = ((noise as f64 / 255.0 - 0.5) * 2.0 * self.intensity * 255.0) as i16;
                    let value = data[idx + i] as i16;
                    data[idx + i] = (value + delta).clamp(0, 255) as u8;
                }
            }
        }
    }

    /// Generate a deterministic "fingerprint" string.
    /// Used when toDataURL or similar is called.
    pub fn generate_data_url_hash(&self, original_hash: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        original_hash.hash(&mut hasher);
        let hash = hasher.finish();

        format!("{:016x}", hash)
    }

    /// Check if a canvas operation should be blocked.
    /// Some operations are too dangerous to allow.
    pub fn should_block_operation(method: &str) -> bool {
        matches!(
            method,
            "getImageData" | "toDataURL" | "toBlob" | "measureText"
        )
    }
}

/// Intercept canvas getImageData and apply noise.
pub fn intercept_get_image_data(
    defense: &CanvasDefense,
    original_data: &mut [u8],
    width: u32,
    height: u32,
) {
    defense.apply_noise(original_data, width, height);
}

/// Generate a fake toDataURL result.
pub fn fake_to_data_url(defense: &CanvasDefense, _content_hint: &str) -> String {
    // Return a data URL that's consistent for this identity
    // but different from the real canvas content
    let hash = defense.generate_data_url_hash("canvas");
    format!(
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=={}",
        &hash[..8]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_application() {
        let defense = CanvasDefense::new(12345);

        let mut data1 = vec![128u8; 16]; // 1x1 pixel RGBA * 4
        let mut data2 = vec![128u8; 16];

        defense.apply_noise(&mut data1, 2, 2);
        defense.apply_noise(&mut data2, 2, 2);

        // Same seed should produce same noise
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_different_seeds_different_noise() {
        let defense1 = CanvasDefense::new(12345);
        let defense2 = CanvasDefense::new(54321);

        let mut data1 = vec![128u8; 16];
        let mut data2 = vec![128u8; 16];

        defense1.apply_noise(&mut data1, 2, 2);
        defense2.apply_noise(&mut data2, 2, 2);

        // Different seeds should produce different noise
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_should_block() {
        assert!(CanvasDefense::should_block_operation("getImageData"));
        assert!(CanvasDefense::should_block_operation("toDataURL"));
        assert!(!CanvasDefense::should_block_operation("fillRect"));
    }
}
