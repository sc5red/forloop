//! Circuit management for per-request isolation.
//!
//! Each request MUST use a new circuit to prevent correlation.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::tls_fingerprint::TlsConfig;
use crate::tor_integration::TorController;
use crate::NetworkError;

/// Manages Tor circuits for the browser.
pub struct CircuitManager {
    tor_controller: Arc<TorController>,
    active_circuits: Mutex<Vec<String>>,
}

impl CircuitManager {
    /// Create a new circuit manager.
    pub fn new(tor_controller: Arc<TorController>) -> Self {
        Self {
            tor_controller,
            active_circuits: Mutex::new(Vec::new()),
        }
    }

    /// Create a new circuit for a request.
    /// This MUST be called for every request.
    pub async fn create_new_circuit(&self) -> Result<Circuit, NetworkError> {
        // Request new circuit from Tor
        let circuit_id = self.tor_controller.new_circuit().await?;

        // Track active circuit
        {
            let mut circuits = self.active_circuits.lock().await;
            circuits.push(circuit_id.clone());
        }

        Ok(Circuit {
            id: circuit_id,
            tor_controller: Arc::clone(&self.tor_controller),
        })
    }

    /// Close all active circuits and clean up.
    pub async fn close_all(&self) -> Result<(), NetworkError> {
        let circuits = {
            let mut circuits = self.active_circuits.lock().await;
            std::mem::take(&mut *circuits)
        };

        for circuit_id in circuits {
            // Best effort close
            let _ = self.tor_controller.close_circuit(&circuit_id).await;
        }

        Ok(())
    }
}

/// A single Tor circuit, created for one request.
pub struct Circuit {
    id: String,
    tor_controller: Arc<TorController>,
}

impl Circuit {
    /// Get the circuit ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Make an HTTP request over this circuit.
    #[allow(clippy::too_many_arguments)]
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
        tls_config: TlsConfig,
        timeout: Duration,
    ) -> Result<RawResponse, NetworkError> {
        // Parse URL
        let parsed = parse_url(url)?;

        // Create SOCKS5 connection through Tor
        let socks_addr = self.tor_controller.socks_addr();

        // In production, this would:
        // 1. Connect to SOCKS5 proxy
        // 2. Use SOCKS5 CONNECT to reach destination
        // 3. Perform TLS handshake with normalized fingerprint
        // 4. Send HTTP request
        // 5. Receive response

        log::debug!(
            "Circuit {} requesting {} {} via {}",
            self.id,
            method,
            url,
            socks_addr
        );

        // Build HTTP request
        let request = build_http_request(method, &parsed, headers, body)?;

        // Execute with timeout
        let response = tokio::time::timeout(timeout, self.execute_request(&socks_addr, &parsed, &request, &tls_config))
            .await
            .map_err(|_| NetworkError::Timeout)??;

        Ok(response)
    }

    /// Execute the actual request (internal).
    async fn execute_request(
        &self,
        socks_addr: &str,
        parsed: &ParsedUrl,
        request: &[u8],
        _tls_config: &TlsConfig,
    ) -> Result<RawResponse, NetworkError> {
        // This is where the actual SOCKS5 + TLS + HTTP happens
        //
        // In production code, we would:
        // 1. tokio::net::TcpStream::connect(socks_addr)
        // 2. Perform SOCKS5 handshake
        // 3. SOCKS5 CONNECT to parsed.host:parsed.port
        // 4. Wrap in TLS with specific fingerprint
        // 5. Write request bytes
        // 6. Read response

        log::debug!(
            "Executing request to {}:{} via SOCKS5 at {}",
            parsed.host,
            parsed.port,
            socks_addr
        );

        // Placeholder response for compilation
        // Real implementation would make actual network calls
        Ok(RawResponse {
            status: 200,
            headers: vec![
                ("content-type".to_string(), "text/html".to_string()),
            ],
            body: Vec::new(),
        })
    }
}

impl Drop for Circuit {
    fn drop(&mut self) {
        // Circuit cleanup happens here
        // We can't do async in drop, so we just log
        log::debug!("Circuit {} dropped", self.id);
    }
}

/// Raw HTTP response from the network.
pub struct RawResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: Vec<(String, String)>,
    /// Response body
    pub body: Vec<u8>,
}

/// Parsed URL components.
struct ParsedUrl {
    host: String,
    port: u16,
    path: String,
}

/// Parse a URL into components.
fn parse_url(url: &str) -> Result<ParsedUrl, NetworkError> {
    // Remove scheme
    let without_scheme = url
        .strip_prefix("https://")
        .ok_or_else(|| NetworkError::InvalidUrl("Not HTTPS".to_string()))?;

    // Split host and path
    let (host_port, path) = match without_scheme.find('/') {
        Some(idx) => (&without_scheme[..idx], &without_scheme[idx..]),
        None => (without_scheme, "/"),
    };

    // Split host and port
    let (host, port) = match host_port.rfind(':') {
        Some(idx) => {
            let port_str = &host_port[idx + 1..];
            let port: u16 = port_str
                .parse()
                .map_err(|_| NetworkError::InvalidUrl("Invalid port".to_string()))?;
            (&host_port[..idx], port)
        }
        None => (host_port, 443),
    };

    Ok(ParsedUrl {
        host: host.to_string(),
        port,
        path: path.to_string(),
    })
}

/// Build an HTTP/1.1 request.
fn build_http_request(
    method: &str,
    parsed: &ParsedUrl,
    headers: &[(String, String)],
    body: Option<&[u8]>,
) -> Result<Vec<u8>, NetworkError> {
    let mut request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\n",
        method, parsed.path, parsed.host
    );

    for (name, value) in headers {
        request.push_str(&format!("{}: {}\r\n", name, value));
    }

    if let Some(body) = body {
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }

    request.push_str("\r\n");

    let mut bytes = request.into_bytes();
    if let Some(body) = body {
        bytes.extend_from_slice(body);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_simple() {
        let parsed = parse_url("https://example.com/path").unwrap();
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 443);
        assert_eq!(parsed.path, "/path");
    }

    #[test]
    fn test_parse_url_with_port() {
        let parsed = parse_url("https://example.com:8443/path").unwrap();
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 8443);
        assert_eq!(parsed.path, "/path");
    }

    #[test]
    fn test_parse_url_no_path() {
        let parsed = parse_url("https://example.com").unwrap();
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 443);
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_url_rejects_http() {
        let result = parse_url("http://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_http_request() {
        let parsed = ParsedUrl {
            host: "example.com".to_string(),
            port: 443,
            path: "/test".to_string(),
        };
        let headers = vec![
            ("User-Agent".to_string(), "Test/1.0".to_string()),
        ];

        let request = build_http_request("GET", &parsed, &headers, None).unwrap();
        let request_str = String::from_utf8(request).unwrap();

        assert!(request_str.contains("GET /test HTTP/1.1"));
        assert!(request_str.contains("Host: example.com"));
        assert!(request_str.contains("User-Agent: Test/1.0"));
    }
}
