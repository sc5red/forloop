//! Integration tests for forloop network layer.
//!
//! These tests verify the privacy guarantees of the network stack.
//! They are designed to be run in an isolated environment with Tor available.

use std::collections::HashSet;
use std::time::Duration;

/// Test that each request gets a different circuit.
#[tokio::test]
#[ignore = "requires Tor daemon"]
async fn test_per_request_circuit_isolation() {
    use forloop_network::{AnonymityLayer, AnonymityConfig};
    
    let config = AnonymityConfig::default();
    let layer = AnonymityLayer::new(config).await.expect("failed to create layer");
    
    // Make multiple requests to a service that shows our IP
    let mut exit_ips = HashSet::new();
    
    for _ in 0..5 {
        let response = layer
            .fetch("https://check.torproject.org/api/ip")
            .await
            .expect("request failed");
        
        let body: serde_json::Value = serde_json::from_slice(&response.body)
            .expect("invalid json");
        
        let ip = body["IP"].as_str().expect("no IP field");
        exit_ips.insert(ip.to_string());
        
        // Small delay to allow circuit rotation
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // We should have multiple different exit IPs
    // Note: Not guaranteed to be 5 different IPs due to exit node reuse,
    // but should be more than 1 most of the time
    assert!(
        exit_ips.len() >= 2,
        "Expected different exit IPs, got: {:?}",
        exit_ips
    );
}

/// Test that headers are properly synthesized.
#[tokio::test]
#[ignore = "requires Tor daemon"]
async fn test_header_synthesis() {
    use forloop_network::{AnonymityLayer, AnonymityConfig};
    
    let config = AnonymityConfig::default();
    let layer = AnonymityLayer::new(config).await.expect("failed to create layer");
    
    // Use httpbin to echo our headers
    let response = layer
        .fetch("https://httpbin.org/headers")
        .await
        .expect("request failed");
    
    let body: serde_json::Value = serde_json::from_slice(&response.body)
        .expect("invalid json");
    
    let headers = &body["headers"];
    
    // Verify User-Agent matches Tor Browser
    let user_agent = headers["User-Agent"].as_str().expect("no User-Agent");
    assert!(
        user_agent.contains("Firefox") && user_agent.contains("Windows"),
        "User-Agent should match Tor Browser pattern: {}",
        user_agent
    );
    
    // Verify no identifying headers leaked
    assert!(headers.get("X-Forwarded-For").is_none(), "X-Forwarded-For should not be present");
    assert!(headers.get("X-Real-Ip").is_none(), "X-Real-IP should not be present");
}

/// Test that request padding is applied.
#[tokio::test]
#[ignore = "requires Tor daemon"]
async fn test_request_padding() {
    use forloop_network::padding::RequestPadder;
    
    let padder = RequestPadder::new();
    
    // Small body should be padded to minimum size
    let small_body = b"hello";
    let padded = padder.pad(small_body);
    
    assert!(
        padded.len() >= 512,
        "Padded body should be at least 512 bytes, got {}",
        padded.len()
    );
    
    // Original content should be preserved
    assert!(padded.starts_with(small_body));
}

/// Test traffic shaper adds jitter.
#[tokio::test]
async fn test_traffic_shaper_jitter() {
    use forloop_network::traffic_shaper::TrafficShaper;
    use std::time::Instant;
    
    let shaper = TrafficShaper::new();
    
    let mut delays = Vec::new();
    
    for _ in 0..10 {
        let start = Instant::now();
        shaper.apply_jitter().await;
        delays.push(start.elapsed());
    }
    
    // Verify delays are not all the same (jitter is working)
    let first_delay = delays[0];
    let all_same = delays.iter().all(|d| *d == first_delay);
    
    assert!(
        !all_same,
        "Jitter should produce variable delays, but all were {:?}",
        first_delay
    );
}

/// Test that timing APIs are fuzzed.
#[tokio::test]
async fn test_timing_fuzzing() {
    use forloop_fingerprint::timing::TimingDefense;
    
    let defense = TimingDefense::new();
    
    // Get multiple "now" values
    let mut times = Vec::new();
    for _ in 0..100 {
        times.push(defense.fuzzed_now());
        std::thread::sleep(std::time::Duration::from_micros(100));
    }
    
    // Verify resolution is reduced (times should cluster)
    let unique_times: HashSet<_> = times.iter().collect();
    
    // With 100ms resolution, we should have far fewer unique values than calls
    assert!(
        unique_times.len() < times.len() / 2,
        "Timing should be quantized, but got {} unique values from {} calls",
        unique_times.len(),
        times.len()
    );
}
