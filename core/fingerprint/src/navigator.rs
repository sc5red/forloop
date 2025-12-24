//! Navigator property spoofing.
//!
//! The navigator object exposes many fingerprinting vectors.
//! We return standardized, privacy-preserving values.

/// Navigator defense configuration.
#[derive(Debug, Clone)]
pub struct NavigatorDefense {
    /// User agent string
    user_agent: String,
    /// Platform string
    platform: String,
    /// Timezone offset (minutes from UTC)
    timezone_offset: i32,
    /// Language
    language: String,
}

impl NavigatorDefense {
    /// Create a new navigator defense with default values.
    pub fn new() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; rv:115.0) Gecko/20100101 Firefox/115.0"
                .to_string(),
            platform: "Win32".to_string(),
            timezone_offset: 0,
            language: "en-US".to_string(),
        }
    }

    /// Create with specific values from synthetic identity.
    pub fn with_identity(
        user_agent: String,
        platform: String,
        timezone_offset: i32,
    ) -> Self {
        Self {
            user_agent,
            platform,
            timezone_offset,
            language: "en-US".to_string(),
        }
    }

    /// Get all navigator properties.
    pub fn get_properties(&self) -> NavigatorProperties {
        NavigatorProperties {
            user_agent: self.user_agent.clone(),
            platform: self.platform.clone(),
            language: self.language.clone(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            app_name: "Netscape".to_string(),
            app_version: "5.0 (Windows)".to_string(),
            app_code_name: "Mozilla".to_string(),
            product: "Gecko".to_string(),
            product_sub: "20100101".to_string(),
            vendor: "".to_string(), // Firefox has empty vendor
            vendor_sub: "".to_string(),
            build_id: "20181001000000".to_string(), // Fixed build ID
            oscpu: self.get_oscpu(),
            cookie_enabled: false, // Cookies are blocked
            do_not_track: None,    // Not sent (ironically identifies)
            pdf_viewer_enabled: true,
            webdriver: false,
            online: true,
            plugins_length: 0,
            mime_types_length: 0,
        }
    }

    /// Get OS/CPU string based on platform.
    fn get_oscpu(&self) -> String {
        match self.platform.as_str() {
            "Win32" => "Windows NT 10.0; Win64; x64".to_string(),
            "Linux x86_64" => "Linux x86_64".to_string(),
            "MacIntel" => "Intel Mac OS X 10.15".to_string(),
            _ => "Windows NT 10.0; Win64; x64".to_string(),
        }
    }

    /// Get timezone offset.
    pub fn timezone_offset(&self) -> i32 {
        self.timezone_offset
    }

    /// Get locale string.
    pub fn locale(&self) -> &str {
        &self.language
    }
}

impl Default for NavigatorDefense {
    fn default() -> Self {
        Self::new()
    }
}

/// All navigator properties.
#[derive(Debug, Clone)]
pub struct NavigatorProperties {
    /// navigator.userAgent
    pub user_agent: String,
    /// navigator.platform
    pub platform: String,
    /// navigator.language
    pub language: String,
    /// navigator.languages
    pub languages: Vec<String>,
    /// navigator.appName
    pub app_name: String,
    /// navigator.appVersion
    pub app_version: String,
    /// navigator.appCodeName
    pub app_code_name: String,
    /// navigator.product
    pub product: String,
    /// navigator.productSub
    pub product_sub: String,
    /// navigator.vendor
    pub vendor: String,
    /// navigator.vendorSub
    pub vendor_sub: String,
    /// navigator.buildID
    pub build_id: String,
    /// navigator.oscpu
    pub oscpu: String,
    /// navigator.cookieEnabled
    pub cookie_enabled: bool,
    /// navigator.doNotTrack (None = not sent)
    pub do_not_track: Option<String>,
    /// navigator.pdfViewerEnabled
    pub pdf_viewer_enabled: bool,
    /// navigator.webdriver
    pub webdriver: bool,
    /// navigator.onLine
    pub online: bool,
    /// navigator.plugins.length
    pub plugins_length: usize,
    /// navigator.mimeTypes.length
    pub mime_types_length: usize,
}

/// Geolocation defense - always fake.
#[derive(Debug, Clone)]
pub struct GeolocationDefense;

impl GeolocationDefense {
    /// Get a fake position.
    /// Returns coordinates from a predefined set to avoid uniqueness.
    pub fn get_fake_position() -> FakePosition {
        // Return position in a large city (non-identifying)
        FakePosition {
            latitude: 51.5074,  // London
            longitude: -0.1278, // London
            accuracy: 10000.0,  // Very imprecise
            altitude: None,
            altitude_accuracy: None,
            heading: None,
            speed: None,
            timestamp: 0, // Will be set by timing defense
        }
    }

    /// Should geolocation always fail?
    pub fn should_fail() -> bool {
        // Return true to always deny geolocation
        true
    }
}

/// Fake geolocation position.
#[derive(Debug, Clone)]
pub struct FakePosition {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Accuracy in meters
    pub accuracy: f64,
    /// Altitude
    pub altitude: Option<f64>,
    /// Altitude accuracy
    pub altitude_accuracy: Option<f64>,
    /// Heading
    pub heading: Option<f64>,
    /// Speed
    pub speed: Option<f64>,
    /// Timestamp
    pub timestamp: u64,
}

/// Permission API responses - always deny.
pub fn get_permission_state(_name: &str) -> &'static str {
    "denied"
}

/// Media devices - always return empty.
pub fn get_media_devices() -> Vec<()> {
    Vec::new()
}

/// Credential API - always fail.
pub fn credentials_available() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigator_properties() {
        let defense = NavigatorDefense::new();
        let props = defense.get_properties();

        assert!(props.user_agent.contains("Firefox"));
        assert!(!props.cookie_enabled);
        assert!(!props.webdriver);
        assert_eq!(props.plugins_length, 0);
    }

    #[test]
    fn test_geolocation_fails() {
        assert!(GeolocationDefense::should_fail());
    }

    #[test]
    fn test_permissions_denied() {
        assert_eq!(get_permission_state("camera"), "denied");
        assert_eq!(get_permission_state("microphone"), "denied");
        assert_eq!(get_permission_state("geolocation"), "denied");
    }
}
