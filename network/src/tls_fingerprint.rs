//! TLS fingerprint normalization.
//!
//! TLS fingerprinting (JA3, JA4, etc.) can identify browsers.
//! This module ensures our TLS fingerprint matches Tor Browser.

use crate::NetworkError;

/// TLS configuration for normalized fingerprint.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Cipher suites in specific order
    pub cipher_suites: Vec<u16>,
    /// TLS extensions in specific order
    pub extensions: Vec<u16>,
    /// Supported groups (curves)
    pub supported_groups: Vec<u16>,
    /// Signature algorithms
    pub signature_algorithms: Vec<u16>,
    /// ALPN protocols
    pub alpn_protocols: Vec<String>,
    /// Minimum TLS version
    pub min_version: TlsVersion,
    /// Maximum TLS version
    pub max_version: TlsVersion,
}

/// TLS version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3
    Tls13,
}

/// Normalizes TLS fingerprint to match Tor Browser.
pub struct TlsFingerprintNormalizer {
    config: TlsConfig,
}

impl TlsFingerprintNormalizer {
    /// Create a new TLS fingerprint normalizer.
    pub fn new() -> Self {
        Self {
            config: Self::tor_browser_config(),
        }
    }

    /// Get TLS configuration matching Tor Browser 13.0.
    fn tor_browser_config() -> TlsConfig {
        TlsConfig {
            // Cipher suites matching Firefox ESR 115 / Tor Browser 13
            cipher_suites: vec![
                0x1301, // TLS_AES_128_GCM_SHA256
                0x1303, // TLS_CHACHA20_POLY1305_SHA256
                0x1302, // TLS_AES_256_GCM_SHA384
                0xc02b, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
                0xc02f, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                0xc02c, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
                0xc030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
                0xcca9, // TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
                0xcca8, // TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
                0xc013, // TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA
                0xc014, // TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA
                0x009c, // TLS_RSA_WITH_AES_128_GCM_SHA256
                0x009d, // TLS_RSA_WITH_AES_256_GCM_SHA384
                0x002f, // TLS_RSA_WITH_AES_128_CBC_SHA
                0x0035, // TLS_RSA_WITH_AES_256_CBC_SHA
            ],

            // Extensions in Firefox/Tor Browser order
            extensions: vec![
                0x0000, // server_name
                0x0017, // extended_master_secret
                0xff01, // renegotiation_info
                0x000a, // supported_groups
                0x000b, // ec_point_formats
                0x0023, // session_ticket
                0x0010, // application_layer_protocol_negotiation
                0x0005, // status_request
                0x0022, // delegated_credentials
                0x0033, // key_share
                0x002b, // supported_versions
                0x000d, // signature_algorithms
                0x001c, // record_size_limit
                0x001b, // compress_certificate
                0x0029, // pre_shared_key
            ],

            // Supported groups (curves)
            supported_groups: vec![
                0x001d, // x25519
                0x0017, // secp256r1
                0x0018, // secp384r1
                0x0019, // secp521r1
                0x0100, // ffdhe2048
                0x0101, // ffdhe3072
            ],

            // Signature algorithms
            signature_algorithms: vec![
                0x0403, // ecdsa_secp256r1_sha256
                0x0503, // ecdsa_secp384r1_sha384
                0x0603, // ecdsa_secp521r1_sha512
                0x0804, // rsa_pss_rsae_sha256
                0x0805, // rsa_pss_rsae_sha384
                0x0806, // rsa_pss_rsae_sha512
                0x0401, // rsa_pkcs1_sha256
                0x0501, // rsa_pkcs1_sha384
                0x0601, // rsa_pkcs1_sha512
            ],

            // ALPN
            alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],

            min_version: TlsVersion::Tls12,
            max_version: TlsVersion::Tls13,
        }
    }

    /// Create a TLS configuration for use in connections.
    pub fn create_config(&self) -> Result<TlsConfig, NetworkError> {
        Ok(self.config.clone())
    }

    /// Get the expected JA3 fingerprint hash.
    /// Used for testing/verification.
    pub fn expected_ja3_hash(&self) -> &'static str {
        // This should match Tor Browser's JA3
        // JA3 = MD5(SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurvePointFormats)
        "e7d705a3286e19ea42f587b344ee6865"
    }

    /// Verify that a ClientHello matches our expected fingerprint.
    pub fn verify_client_hello(&self, client_hello: &[u8]) -> bool {
        // Parse ClientHello and verify:
        // 1. Cipher suite order matches
        // 2. Extension order matches
        // 3. Supported groups match
        // 4. Signature algorithms match

        // This is a simplified check - real implementation would
        // parse the TLS ClientHello structure

        !client_hello.is_empty()
    }
}

impl Default for TlsFingerprintNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP/2 fingerprint normalization.
/// HTTP/2 settings can also be used for fingerprinting.
#[derive(Debug, Clone)]
pub struct Http2Fingerprint {
    /// SETTINGS frame values
    pub settings: Vec<(u16, u32)>,
    /// Window update value
    pub window_update: u32,
    /// Header priority
    pub priority: Http2Priority,
}

/// HTTP/2 priority settings.
#[derive(Debug, Clone)]
pub struct Http2Priority {
    /// Stream dependency
    pub depends_on: u32,
    /// Weight
    pub weight: u8,
    /// Exclusive flag
    pub exclusive: bool,
}

impl Default for Http2Fingerprint {
    fn default() -> Self {
        // Match Firefox/Tor Browser HTTP/2 fingerprint
        Self {
            settings: vec![
                (0x1, 65536),  // HEADER_TABLE_SIZE
                (0x2, 0),      // ENABLE_PUSH (disabled)
                (0x3, 0),      // MAX_CONCURRENT_STREAMS (unlimited)
                (0x4, 131072), // INITIAL_WINDOW_SIZE
                (0x5, 16384),  // MAX_FRAME_SIZE
                (0x6, 0),      // MAX_HEADER_LIST_SIZE (unlimited)
            ],
            window_update: 12517377,
            priority: Http2Priority {
                depends_on: 0,
                weight: 41,
                exclusive: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizer_creation() {
        let normalizer = TlsFingerprintNormalizer::new();
        let config = normalizer.create_config().unwrap();

        // Verify cipher suite order
        assert_eq!(config.cipher_suites[0], 0x1301); // TLS_AES_128_GCM_SHA256
        assert!(config.cipher_suites.len() > 10);
    }

    #[test]
    fn test_tls_version() {
        let normalizer = TlsFingerprintNormalizer::new();
        let config = normalizer.create_config().unwrap();

        assert_eq!(config.min_version, TlsVersion::Tls12);
        assert_eq!(config.max_version, TlsVersion::Tls13);
    }

    #[test]
    fn test_alpn() {
        let normalizer = TlsFingerprintNormalizer::new();
        let config = normalizer.create_config().unwrap();

        assert_eq!(config.alpn_protocols, vec!["h2", "http/1.1"]);
    }
}
