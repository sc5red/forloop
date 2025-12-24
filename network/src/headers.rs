//! HTTP Header Synthesis for anonymization.
//!
//! This module generates synthetic HTTP headers that:
//! - Match Tor Browser's fingerprint
//! - Rotate per request where safe
//! - Reveal no identifying information
//! - Use a minimal header set

use rand::seq::SliceRandom;
use rand::Rng;

/// Pre-defined User-Agent strings that match Tor Browser.
/// These MUST be kept in sync with actual Tor Browser releases.
const USER_AGENTS: &[&str] = &[
    // Tor Browser 13.0 on Windows
    "Mozilla/5.0 (Windows NT 10.0; rv:115.0) Gecko/20100101 Firefox/115.0",
    // Tor Browser 13.0 on Linux
    "Mozilla/5.0 (X11; Linux x86_64; rv:115.0) Gecko/20100101 Firefox/115.0",
    // Tor Browser 13.0 on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:115.0) Gecko/20100101 Firefox/115.0",
];

/// Accept-Language values - kept generic and common.
const ACCEPT_LANGUAGES: &[&str] = &[
    "en-US,en;q=0.5",
];

/// Accept header for HTML pages.
const ACCEPT_HTML: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8";

/// Accept header for images.
const ACCEPT_IMAGE: &str = "image/avif,image/webp,*/*";

/// Accept-Encoding header.
const ACCEPT_ENCODING: &str = "gzip, deflate, br";

/// Synthesizes HTTP headers for anonymized requests.
pub struct HeaderSynthesizer {
    /// Random number generator
    rng: std::sync::Mutex<rand::rngs::ThreadRng>,
}

impl HeaderSynthesizer {
    /// Create a new header synthesizer.
    pub fn new() -> Self {
        Self {
            rng: std::sync::Mutex::new(rand::thread_rng()),
        }
    }

    /// Generate a complete set of synthetic headers for a request.
    pub fn generate(&self) -> SyntheticHeaders {
        let mut rng = self.rng.lock().expect("RNG lock poisoned");

        // Select User-Agent (rotated per request)
        let user_agent = USER_AGENTS
            .choose(&mut *rng)
            .expect("USER_AGENTS is non-empty")
            .to_string();

        // Accept-Language is fixed (variation would fingerprint)
        let accept_language = ACCEPT_LANGUAGES[0].to_string();

        SyntheticHeaders {
            user_agent,
            accept: ACCEPT_HTML.to_string(),
            accept_language,
            accept_encoding: ACCEPT_ENCODING.to_string(),
        }
    }

    /// Generate headers for an image request.
    pub fn generate_for_image(&self) -> SyntheticHeaders {
        let mut headers = self.generate();
        headers.accept = ACCEPT_IMAGE.to_string();
        headers
    }

    /// Convert synthetic headers to a list of (name, value) pairs.
    pub fn to_header_list(headers: &SyntheticHeaders) -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), headers.user_agent.clone()),
            ("Accept".to_string(), headers.accept.clone()),
            ("Accept-Language".to_string(), headers.accept_language.clone()),
            ("Accept-Encoding".to_string(), headers.accept_encoding.clone()),
            ("Connection".to_string(), "keep-alive".to_string()),
            ("Upgrade-Insecure-Requests".to_string(), "1".to_string()),
            ("Sec-Fetch-Dest".to_string(), "document".to_string()),
            ("Sec-Fetch-Mode".to_string(), "navigate".to_string()),
            ("Sec-Fetch-Site".to_string(), "none".to_string()),
            ("Sec-Fetch-User".to_string(), "?1".to_string()),
            // Explicitly NOT sending:
            // - Referer (tracking)
            // - Cookie (tracking)
            // - DNT (ironically identifies privacy users)
            // - X-Forwarded-For (internal only)
            // - Any custom headers
        ]
    }
}

impl Default for HeaderSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of synthetic HTTP headers.
#[derive(Debug, Clone)]
pub struct SyntheticHeaders {
    /// User-Agent header
    pub user_agent: String,
    /// Accept header
    pub accept: String,
    /// Accept-Language header
    pub accept_language: String,
    /// Accept-Encoding header
    pub accept_encoding: String,
}

impl SyntheticHeaders {
    /// Convert to header list for use in requests.
    pub fn to_vec(&self) -> Vec<(String, String)> {
        HeaderSynthesizer::to_header_list(self)
    }
}

/// Strips dangerous headers from outgoing requests.
/// Used as a last line of defense.
pub fn strip_dangerous_headers(headers: &mut Vec<(String, String)>) {
    let dangerous = [
        "cookie",
        "authorization",
        "proxy-authorization",
        "x-forwarded-for",
        "x-real-ip",
        "x-client-ip",
        "forwarded",
        "via",
        "x-request-id",
        "x-correlation-id",
        "dnt",
        "referer",
        "origin", // Except for CORS, but we don't do cross-origin
    ];

    headers.retain(|(name, _)| {
        let lower = name.to_lowercase();
        !dangerous.contains(&lower.as_str())
    });
}

/// Normalizes header order to match Tor Browser.
/// Header order can be used for fingerprinting.
pub fn normalize_header_order(headers: &mut Vec<(String, String)>) {
    // Tor Browser/Firefox header order
    let order = [
        "host",
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        "connection",
        "upgrade-insecure-requests",
        "sec-fetch-dest",
        "sec-fetch-mode",
        "sec-fetch-site",
        "sec-fetch-user",
        "content-type",
        "content-length",
    ];

    headers.sort_by(|(a, _), (b, _)| {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        let a_pos = order.iter().position(|&x| x == a_lower).unwrap_or(usize::MAX);
        let b_pos = order.iter().position(|&x| x == b_lower).unwrap_or(usize::MAX);

        a_pos.cmp(&b_pos)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_creates_valid_headers() {
        let synth = HeaderSynthesizer::new();
        let headers = synth.generate();

        assert!(!headers.user_agent.is_empty());
        assert!(headers.user_agent.contains("Firefox"));
        assert!(headers.accept_language.starts_with("en"));
    }

    #[test]
    fn test_dangerous_header_stripping() {
        let mut headers = vec![
            ("User-Agent".to_string(), "Test".to_string()),
            ("Cookie".to_string(), "bad=tracking".to_string()),
            ("Referer".to_string(), "https://previous.site".to_string()),
            ("Accept".to_string(), "text/html".to_string()),
        ];

        strip_dangerous_headers(&mut headers);

        assert_eq!(headers.len(), 2);
        assert!(headers.iter().any(|(n, _)| n == "User-Agent"));
        assert!(headers.iter().any(|(n, _)| n == "Accept"));
        assert!(!headers.iter().any(|(n, _)| n.to_lowercase() == "cookie"));
        assert!(!headers.iter().any(|(n, _)| n.to_lowercase() == "referer"));
    }

    #[test]
    fn test_header_order_normalization() {
        let mut headers = vec![
            ("Accept".to_string(), "text/html".to_string()),
            ("Host".to_string(), "example.com".to_string()),
            ("User-Agent".to_string(), "Test".to_string()),
        ];

        normalize_header_order(&mut headers);

        assert_eq!(headers[0].0, "Host");
        assert_eq!(headers[1].0, "User-Agent");
        assert_eq!(headers[2].0, "Accept");
    }
}
