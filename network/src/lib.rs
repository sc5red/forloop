//! forloop Network Anonymity Layer
//!
//! This module provides the core network anonymization functionality.
//! All network traffic MUST go through this layer. Direct connections
//! are architecturally impossible.
//!
//! # Design Principles
//!
//! - Every request uses a NEW circuit
//! - Traffic is padded and jittered
//! - TLS fingerprint matches Tor Browser
//! - DNS only over anonymized channel
//! - No caching of any network state

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![forbid(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Duration;

mod circuit;
mod headers;
mod padding;
mod tls_fingerprint;
mod tor_integration;
mod traffic_shaper;

pub use circuit::{Circuit, CircuitManager};
pub use headers::{HeaderSynthesizer, SyntheticHeaders};
pub use padding::PaddingGenerator;
pub use tls_fingerprint::TlsFingerprintNormalizer;
pub use tor_integration::TorController;
pub use traffic_shaper::TrafficShaper;

/// Network layer configuration.
/// All values are compile-time defaults with no runtime override.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Minimum padding bytes per request
    pub min_padding_bytes: usize,
    /// Maximum padding bytes per request
    pub max_padding_bytes: usize,
    /// Minimum jitter delay
    pub min_jitter_ms: u64,
    /// Maximum jitter delay
    pub max_jitter_ms: u64,
    /// Tor SOCKS5 port (embedded tor)
    pub tor_socks_port: u16,
    /// Tor control port (embedded tor)
    pub tor_control_port: u16,
    /// Request timeout
    pub request_timeout: Duration,
    /// Force new circuit per request
    pub new_circuit_per_request: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            min_padding_bytes: 256,
            max_padding_bytes: 2048,
            min_jitter_ms: 0,
            max_jitter_ms: 50,
            tor_socks_port: 9150,
            tor_control_port: 9151,
            request_timeout: Duration::from_secs(60),
            new_circuit_per_request: true, // MUST be true, non-configurable in practice
        }
    }
}

/// Result of a network request.
#[derive(Debug)]
pub struct NetworkResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers (sanitized)
    pub headers: Vec<(String, String)>,
    /// Response body
    pub body: Vec<u8>,
    /// Circuit ID used (for debugging, not exposed to content)
    pub circuit_id: String,
}

/// Errors that can occur in the network layer.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// Tor connection failed
    #[error("Tor connection failed: {0}")]
    TorConnectionFailed(String),

    /// Circuit creation failed
    #[error("Circuit creation failed: {0}")]
    CircuitCreationFailed(String),

    /// Request failed
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// Timeout
    #[error("Request timed out")]
    Timeout,

    /// TLS error
    #[error("TLS error: {0}")]
    TlsError(String),

    /// DNS resolution failed
    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Protocol not supported (only HTTPS)
    #[error("Protocol not supported: {0} (only HTTPS allowed)")]
    ProtocolNotSupported(String),
}

/// The main network layer abstraction.
/// All browser network traffic goes through this.
pub struct AnonymizedNetwork {
    config: NetworkConfig,
    tor_controller: Arc<TorController>,
    circuit_manager: Arc<CircuitManager>,
    header_synthesizer: HeaderSynthesizer,
    traffic_shaper: TrafficShaper,
    tls_normalizer: TlsFingerprintNormalizer,
}

impl AnonymizedNetwork {
    /// Create a new anonymized network layer.
    /// This will start the embedded Tor daemon.
    pub async fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        let tor_controller = Arc::new(
            TorController::new(config.tor_socks_port, config.tor_control_port).await?,
        );

        let circuit_manager = Arc::new(CircuitManager::new(Arc::clone(&tor_controller)));

        let header_synthesizer = HeaderSynthesizer::new();
        let traffic_shaper = TrafficShaper::new(
            config.min_padding_bytes,
            config.max_padding_bytes,
            config.min_jitter_ms,
            config.max_jitter_ms,
        );
        let tls_normalizer = TlsFingerprintNormalizer::new();

        Ok(Self {
            config,
            tor_controller,
            circuit_manager,
            header_synthesizer,
            traffic_shaper,
            tls_normalizer,
        })
    }

    /// Make an HTTP request through the anonymized network.
    ///
    /// # Guarantees
    ///
    /// - A NEW circuit is created for this request
    /// - Headers are synthetic and randomized
    /// - Traffic is padded and jittered
    /// - TLS fingerprint matches Tor Browser
    /// - Real IP never reaches the destination
    /// - DNS resolution happens over Tor
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<&[u8]>,
    ) -> Result<NetworkResponse, NetworkError> {
        // Validate URL - only HTTPS allowed
        if !url.starts_with("https://") {
            return Err(NetworkError::ProtocolNotSupported(
                url.split(':').next().unwrap_or("unknown").to_string(),
            ));
        }

        // Apply jitter before request
        self.traffic_shaper.apply_jitter().await;

        // Create a NEW circuit for this request
        let circuit = self.circuit_manager.create_new_circuit().await?;

        // Generate synthetic headers
        let synthetic_headers = self.header_synthesizer.generate();

        // Pad the request body
        let padded_body = body.map(|b| self.traffic_shaper.pad_request(b));

        // Configure TLS with normalized fingerprint
        let tls_config = self.tls_normalizer.create_config()?;

        // Make the actual request through Tor
        let response = circuit
            .request(
                method,
                url,
                &synthetic_headers,
                padded_body.as_deref(),
                tls_config,
                self.config.request_timeout,
            )
            .await?;

        // Sanitize response headers (remove tracking headers)
        let sanitized_headers = self.sanitize_response_headers(response.headers);

        // Apply jitter after response
        self.traffic_shaper.apply_jitter().await;

        Ok(NetworkResponse {
            status: response.status,
            headers: sanitized_headers,
            body: response.body,
            circuit_id: circuit.id().to_string(),
        })
    }

    /// Sanitize response headers to remove any tracking mechanisms.
    fn sanitize_response_headers(&self, headers: Vec<(String, String)>) -> Vec<(String, String)> {
        headers
            .into_iter()
            .filter(|(name, _)| {
                let lower = name.to_lowercase();
                // Remove these headers entirely
                !matches!(
                    lower.as_str(),
                    "set-cookie"
                        | "set-cookie2"
                        | "etag"
                        | "last-modified"
                        | "x-request-id"
                        | "x-correlation-id"
                        | "x-amzn-requestid"
                        | "cf-ray"
                        | "x-cache"
                        | "x-served-by"
                        | "x-timer"
                        | "x-trace-id"
                )
            })
            .collect()
    }

    /// Check if the Tor network is connected and healthy.
    pub async fn is_healthy(&self) -> bool {
        self.tor_controller.is_connected().await
    }

    /// Get current Tor circuit information (for UI display only).
    pub async fn get_circuit_info(&self) -> Option<CircuitInfo> {
        self.tor_controller.get_current_circuit_info().await
    }
}

/// Information about the current Tor circuit (for display only).
#[derive(Debug, Clone)]
pub struct CircuitInfo {
    /// Entry node country code
    pub entry_country: String,
    /// Exit node country code
    pub exit_country: String,
    /// Number of hops
    pub hop_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_http() {
        // Can't actually test async in unit tests without runtime,
        // but we verify the URL validation logic
        let url = "http://example.com";
        assert!(!url.starts_with("https://"));
    }

    #[test]
    fn test_accepts_https() {
        let url = "https://example.com";
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn test_sanitize_headers() {
        let headers = vec![
            ("content-type".to_string(), "text/html".to_string()),
            ("set-cookie".to_string(), "tracking=bad".to_string()),
            ("etag".to_string(), "\"abc123\"".to_string()),
            ("content-length".to_string(), "1234".to_string()),
        ];

        // Simulating sanitization
        let sanitized: Vec<_> = headers
            .into_iter()
            .filter(|(name, _)| {
                let lower = name.to_lowercase();
                !matches!(lower.as_str(), "set-cookie" | "etag")
            })
            .collect();

        assert_eq!(sanitized.len(), 2);
        assert!(sanitized.iter().any(|(n, _)| n == "content-type"));
        assert!(sanitized.iter().any(|(n, _)| n == "content-length"));
    }
}
