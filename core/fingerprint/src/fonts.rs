//! Font enumeration defense.
//!
//! Font enumeration reveals installed fonts, which are highly unique.
//! We expose only a fixed set of web-safe fonts.

/// The fixed set of fonts exposed to websites.
/// These are common system fonts that don't reveal user information.
pub const ALLOWED_FONTS: &[&str] = &[
    // Basic fonts present on all systems
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    // Common cross-platform fonts
    "Arial",
    "Helvetica",
    "Times New Roman",
    "Times",
    "Courier New",
    "Courier",
    "Georgia",
    "Verdana",
    "Trebuchet MS",
];

/// Font defense configuration.
#[derive(Debug, Clone)]
pub struct FontDefense {
    /// Allowed fonts
    allowed_fonts: Vec<String>,
}

impl FontDefense {
    /// Create a new font defense with default fonts.
    pub fn new() -> Self {
        Self {
            allowed_fonts: ALLOWED_FONTS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Check if a font is allowed.
    pub fn is_font_allowed(&self, font_name: &str) -> bool {
        let normalized = font_name.trim().to_lowercase();
        self.allowed_fonts
            .iter()
            .any(|f| f.to_lowercase() == normalized)
    }

    /// Get the list of allowed fonts.
    pub fn allowed_fonts(&self) -> &[String] {
        &self.allowed_fonts
    }

    /// Filter a font list to only allowed fonts.
    pub fn filter_fonts(&self, fonts: &[String]) -> Vec<String> {
        fonts
            .iter()
            .filter(|f| self.is_font_allowed(f))
            .cloned()
            .collect()
    }

    /// Get font metrics for a given font.
    ///
    /// Returns standardized metrics to prevent fingerprinting via
    /// font rendering differences.
    pub fn get_font_metrics(&self, font_name: &str, font_size: f32) -> FontMetrics {
        // Return consistent metrics regardless of actual font
        let base_height = font_size * 1.2;
        let base_width = font_size * 0.6;

        FontMetrics {
            height: base_height,
            ascent: font_size * 0.8,
            descent: font_size * 0.2,
            line_gap: font_size * 0.2,
            average_char_width: base_width,
            max_char_width: base_width * 1.5,
            x_height: font_size * 0.5,
            cap_height: font_size * 0.7,
        }
    }

    /// Check if a CSS font-family value should be modified.
    pub fn sanitize_font_family(&self, css_value: &str) -> String {
        // Parse the font-family value and filter to allowed fonts
        let fonts: Vec<&str> = css_value.split(',').map(|s| s.trim()).collect();

        let filtered: Vec<&str> = fonts
            .into_iter()
            .filter(|f| {
                let name = f.trim_matches(|c| c == '"' || c == '\'');
                self.is_font_allowed(name)
            })
            .collect();

        if filtered.is_empty() {
            // Default to sans-serif if no fonts allowed
            "sans-serif".to_string()
        } else {
            filtered.join(", ")
        }
    }
}

impl Default for FontDefense {
    fn default() -> Self {
        Self::new()
    }
}

/// Standardized font metrics.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Total line height
    pub height: f32,
    /// Ascent (above baseline)
    pub ascent: f32,
    /// Descent (below baseline)
    pub descent: f32,
    /// Gap between lines
    pub line_gap: f32,
    /// Average character width
    pub average_char_width: f32,
    /// Maximum character width
    pub max_char_width: f32,
    /// x-height (height of lowercase x)
    pub x_height: f32,
    /// Cap height (height of capital letters)
    pub cap_height: f32,
}

/// Block font enumeration APIs.
///
/// These methods should return empty or null when called by JS.
pub fn blocked_font_apis() -> &'static [&'static str] {
    &[
        "fonts.check",
        "fonts.load",
        "fonts.ready",
        "document.fonts",
        "FontFaceSet",
        "FontFace",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_fonts() {
        let defense = FontDefense::new();

        assert!(defense.is_font_allowed("Arial"));
        assert!(defense.is_font_allowed("arial")); // Case insensitive
        assert!(defense.is_font_allowed(" Arial ")); // Whitespace tolerant
        assert!(!defense.is_font_allowed("Comic Sans MS"));
        assert!(!defense.is_font_allowed("CustomFont"));
    }

    #[test]
    fn test_filter_fonts() {
        let defense = FontDefense::new();
        let fonts = vec![
            "Arial".to_string(),
            "Comic Sans MS".to_string(),
            "Verdana".to_string(),
        ];

        let filtered = defense.filter_fonts(&fonts);
        assert_eq!(filtered, vec!["Arial", "Verdana"]);
    }

    #[test]
    fn test_sanitize_font_family() {
        let defense = FontDefense::new();

        assert_eq!(
            defense.sanitize_font_family("Arial, 'Comic Sans MS', sans-serif"),
            "Arial, sans-serif"
        );

        assert_eq!(
            defense.sanitize_font_family("'Unknown Font', 'Another Unknown'"),
            "sans-serif"
        );
    }

    #[test]
    fn test_font_metrics_consistency() {
        let defense = FontDefense::new();

        let metrics1 = defense.get_font_metrics("Arial", 16.0);
        let metrics2 = defense.get_font_metrics("Verdana", 16.0);

        // All fonts return same metrics (normalized)
        assert_eq!(metrics1.height, metrics2.height);
    }
}
