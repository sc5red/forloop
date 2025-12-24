//! Hardware property spoofing.
//!
//! Hardware properties like CPU cores and memory are fingerprinting vectors.
//! We return values from a defined anonymity set.

use rand::Rng;

/// Hardware profile with spoofed values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareProfile {
    /// Number of logical CPU cores
    pub hardware_concurrency: u8,
    /// Device memory in GB
    pub device_memory: u8,
    /// Max touch points
    pub max_touch_points: u8,
}

impl HardwareProfile {
    /// Pre-defined hardware profiles representing common configurations.
    pub const PROFILES: &'static [HardwareProfile] = &[
        // Common laptop
        HardwareProfile {
            hardware_concurrency: 4,
            device_memory: 8,
            max_touch_points: 0,
        },
        // Mid-range desktop
        HardwareProfile {
            hardware_concurrency: 8,
            device_memory: 8,
            max_touch_points: 0,
        },
        // Higher-end
        HardwareProfile {
            hardware_concurrency: 8,
            device_memory: 16,
            max_touch_points: 0,
        },
        // Basic laptop
        HardwareProfile {
            hardware_concurrency: 2,
            device_memory: 4,
            max_touch_points: 0,
        },
    ];

    /// Select a random hardware profile.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let idx = rng.gen_range(0..Self::PROFILES.len());
        Self::PROFILES[idx].clone()
    }

    /// Get the default profile.
    pub fn default_profile() -> Self {
        Self::PROFILES[0].clone()
    }
}

/// Hardware defense configuration.
#[derive(Debug, Clone)]
pub struct HardwareDefense {
    /// Selected hardware profile
    profile: HardwareProfile,
}

impl HardwareDefense {
    /// Create a new hardware defense.
    pub fn new(profile: HardwareProfile) -> Self {
        Self { profile }
    }

    /// Create with default profile.
    pub fn default_defense() -> Self {
        Self {
            profile: HardwareProfile::default_profile(),
        }
    }

    /// Get spoofed hardware concurrency.
    pub fn hardware_concurrency(&self) -> u8 {
        self.profile.hardware_concurrency
    }

    /// Get spoofed device memory.
    pub fn device_memory(&self) -> u8 {
        self.profile.device_memory
    }

    /// Get max touch points (0 = no touch).
    pub fn max_touch_points(&self) -> u8 {
        self.profile.max_touch_points
    }

    /// Get all hardware properties.
    pub fn get_properties(&self) -> HardwareProperties {
        HardwareProperties {
            hardware_concurrency: self.hardware_concurrency(),
            device_memory: self.device_memory(),
            max_touch_points: self.max_touch_points(),
            // Always return false for these
            bluetooth_available: false,
            usb_available: false,
            nfc_available: false,
            midi_available: false,
            hid_available: false,
            serial_available: false,
            battery_available: false,
            geolocation_available: false,
            accelerometer_available: false,
            gyroscope_available: false,
            magnetometer_available: false,
            ambient_light_available: false,
        }
    }
}

impl Default for HardwareDefense {
    fn default() -> Self {
        Self::default_defense()
    }
}

/// All hardware properties exposed to websites.
#[derive(Debug, Clone)]
pub struct HardwareProperties {
    /// Navigator.hardwareConcurrency
    pub hardware_concurrency: u8,
    /// Navigator.deviceMemory
    pub device_memory: u8,
    /// Navigator.maxTouchPoints
    pub max_touch_points: u8,
    /// Bluetooth API available
    pub bluetooth_available: bool,
    /// USB API available
    pub usb_available: bool,
    /// NFC API available
    pub nfc_available: bool,
    /// MIDI API available
    pub midi_available: bool,
    /// HID API available
    pub hid_available: bool,
    /// Serial API available
    pub serial_available: bool,
    /// Battery API available
    pub battery_available: bool,
    /// Geolocation available
    pub geolocation_available: bool,
    /// Accelerometer available
    pub accelerometer_available: bool,
    /// Gyroscope available
    pub gyroscope_available: bool,
    /// Magnetometer available
    pub magnetometer_available: bool,
    /// Ambient light sensor available
    pub ambient_light_available: bool,
}

/// APIs that should be completely blocked/undefined.
pub fn blocked_hardware_apis() -> &'static [&'static str] {
    &[
        "navigator.bluetooth",
        "navigator.usb",
        "navigator.nfc",
        "navigator.hid",
        "navigator.serial",
        "navigator.requestMIDIAccess",
        "navigator.getBattery",
        "navigator.getGamepads",
        "navigator.xr",
        "navigator.keyboard",
        "navigator.wakeLock",
        "navigator.virtualKeyboard",
        "Accelerometer",
        "Gyroscope",
        "Magnetometer",
        "AmbientLightSensor",
        "DeviceMotionEvent",
        "DeviceOrientationEvent",
    ]
}

/// Connection type values - we return a generic value.
#[derive(Debug, Clone)]
pub struct NetworkInformation {
    /// Effective connection type
    pub effective_type: &'static str,
    /// Downlink speed
    pub downlink: f64,
    /// Round-trip time
    pub rtt: u32,
    /// Save data mode
    pub save_data: bool,
}

impl Default for NetworkInformation {
    fn default() -> Self {
        Self {
            effective_type: "4g",
            downlink: 10.0,
            rtt: 100,
            save_data: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_selection() {
        let profile = HardwareProfile::default_profile();
        assert_eq!(profile.hardware_concurrency, 4);
        assert_eq!(profile.device_memory, 8);
    }

    #[test]
    fn test_defense() {
        let defense = HardwareDefense::default_defense();
        assert_eq!(defense.hardware_concurrency(), 4);

        let props = defense.get_properties();
        assert!(!props.bluetooth_available);
        assert!(!props.battery_available);
    }

    #[test]
    fn test_all_sensors_blocked() {
        let defense = HardwareDefense::default_defense();
        let props = defense.get_properties();

        assert!(!props.accelerometer_available);
        assert!(!props.gyroscope_available);
        assert!(!props.magnetometer_available);
        assert!(!props.ambient_light_available);
    }
}
