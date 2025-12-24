//! Screen and window size normalization.
//!
//! Screen dimensions are a fingerprinting vector.
//! We bucket sizes into common values.

use rand::Rng;

/// Common screen size buckets.
/// These represent popular display configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenBucket {
    /// Screen width
    pub width: u32,
    /// Screen height
    pub height: u32,
    /// Color depth
    pub color_depth: u8,
    /// Device pixel ratio
    pub device_pixel_ratio: u8,
}

impl ScreenBucket {
    /// Available screen buckets.
    pub const BUCKETS: &'static [ScreenBucket] = &[
        // Full HD (most common)
        ScreenBucket {
            width: 1920,
            height: 1080,
            color_depth: 24,
            device_pixel_ratio: 1,
        },
        // HD
        ScreenBucket {
            width: 1366,
            height: 768,
            color_depth: 24,
            device_pixel_ratio: 1,
        },
        // WXGA+
        ScreenBucket {
            width: 1440,
            height: 900,
            color_depth: 24,
            device_pixel_ratio: 1,
        },
        // Full HD at 125%
        ScreenBucket {
            width: 1536,
            height: 864,
            color_depth: 24,
            device_pixel_ratio: 1,
        },
        // MacBook-like
        ScreenBucket {
            width: 1280,
            height: 800,
            color_depth: 24,
            device_pixel_ratio: 2,
        },
    ];

    /// Select a random screen bucket.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let idx = rng.gen_range(0..Self::BUCKETS.len());
        Self::BUCKETS[idx]
    }

    /// Get the nearest bucket for actual dimensions.
    pub fn nearest(actual_width: u32, actual_height: u32) -> Self {
        Self::BUCKETS
            .iter()
            .min_by_key(|b| {
                let dw = (b.width as i32 - actual_width as i32).abs();
                let dh = (b.height as i32 - actual_height as i32).abs();
                dw + dh
            })
            .copied()
            .unwrap_or(Self::BUCKETS[0])
    }
}

/// Screen defense configuration.
#[derive(Debug, Clone)]
pub struct ScreenDefense {
    /// Selected screen bucket
    bucket: ScreenBucket,
}

impl ScreenDefense {
    /// Create a new screen defense with a specific bucket.
    pub fn new(bucket: ScreenBucket) -> Self {
        Self { bucket }
    }

    /// Create with a random bucket.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            bucket: ScreenBucket::random(&mut rng),
        }
    }

    /// Get spoofed screen width.
    pub fn screen_width(&self) -> u32 {
        self.bucket.width
    }

    /// Get spoofed screen height.
    pub fn screen_height(&self) -> u32 {
        self.bucket.height
    }

    /// Get spoofed available width (same as screen).
    pub fn avail_width(&self) -> u32 {
        self.bucket.width
    }

    /// Get spoofed available height (slightly less for taskbar).
    pub fn avail_height(&self) -> u32 {
        self.bucket.height - 40 // Standard taskbar height
    }

    /// Get color depth.
    pub fn color_depth(&self) -> u8 {
        self.bucket.color_depth
    }

    /// Get pixel depth (same as color depth).
    pub fn pixel_depth(&self) -> u8 {
        self.bucket.color_depth
    }

    /// Get device pixel ratio.
    pub fn device_pixel_ratio(&self) -> f64 {
        self.bucket.device_pixel_ratio as f64
    }

    /// Get inner window width (letterboxed to match screen).
    pub fn inner_width(&self) -> u32 {
        // Browser window typically has some margin
        self.bucket.width
    }

    /// Get inner window height.
    pub fn inner_height(&self) -> u32 {
        // Account for browser chrome
        self.bucket.height - 100
    }

    /// Get outer window width.
    pub fn outer_width(&self) -> u32 {
        self.bucket.width
    }

    /// Get outer window height.
    pub fn outer_height(&self) -> u32 {
        self.bucket.height
    }

    /// Get screen X position.
    pub fn screen_x(&self) -> i32 {
        0
    }

    /// Get screen Y position.
    pub fn screen_y(&self) -> i32 {
        0
    }

    /// Get all screen properties as a struct.
    pub fn get_screen_properties(&self) -> ScreenProperties {
        ScreenProperties {
            width: self.screen_width(),
            height: self.screen_height(),
            avail_width: self.avail_width(),
            avail_height: self.avail_height(),
            color_depth: self.color_depth(),
            pixel_depth: self.pixel_depth(),
            orientation_type: "landscape-primary".to_string(),
            orientation_angle: 0,
        }
    }

    /// Get all window properties.
    pub fn get_window_properties(&self) -> WindowProperties {
        WindowProperties {
            inner_width: self.inner_width(),
            inner_height: self.inner_height(),
            outer_width: self.outer_width(),
            outer_height: self.outer_height(),
            screen_x: self.screen_x(),
            screen_y: self.screen_y(),
            device_pixel_ratio: self.device_pixel_ratio(),
        }
    }
}

/// Spoofed screen properties.
#[derive(Debug, Clone)]
pub struct ScreenProperties {
    /// Screen width
    pub width: u32,
    /// Screen height
    pub height: u32,
    /// Available width
    pub avail_width: u32,
    /// Available height
    pub avail_height: u32,
    /// Color depth
    pub color_depth: u8,
    /// Pixel depth
    pub pixel_depth: u8,
    /// Orientation type
    pub orientation_type: String,
    /// Orientation angle
    pub orientation_angle: u16,
}

/// Spoofed window properties.
#[derive(Debug, Clone)]
pub struct WindowProperties {
    /// Inner width
    pub inner_width: u32,
    /// Inner height
    pub inner_height: u32,
    /// Outer width
    pub outer_width: u32,
    /// Outer height
    pub outer_height: u32,
    /// Screen X
    pub screen_x: i32,
    /// Screen Y
    pub screen_y: i32,
    /// Device pixel ratio
    pub device_pixel_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_selection() {
        let bucket = ScreenBucket::nearest(1920, 1080);
        assert_eq!(bucket.width, 1920);
        assert_eq!(bucket.height, 1080);

        let bucket = ScreenBucket::nearest(1900, 1000);
        assert_eq!(bucket.width, 1920); // Nearest
    }

    #[test]
    fn test_screen_defense() {
        let defense = ScreenDefense::new(ScreenBucket::BUCKETS[0]);

        assert_eq!(defense.screen_width(), 1920);
        assert_eq!(defense.screen_height(), 1080);
        assert_eq!(defense.color_depth(), 24);
    }

    #[test]
    fn test_avail_height_less_than_screen() {
        let defense = ScreenDefense::new(ScreenBucket::BUCKETS[0]);
        assert!(defense.avail_height() < defense.screen_height());
    }
}
