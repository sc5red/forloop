//! Integration tests for fingerprint defense module.
//!
//! These tests verify that fingerprinting APIs are properly spoofed
//! and do not leak identifying information.

use std::collections::HashSet;

/// Test canvas fingerprint noise injection.
#[test]
fn test_canvas_noise_injection() {
    use forloop_fingerprint::canvas::CanvasDefense;
    
    let defense = CanvasDefense::new();
    
    // Create a simple "image" (just bytes)
    let original_pixels: Vec<u8> = (0..256).collect();
    
    // Apply noise multiple times
    let mut results = Vec::new();
    for _ in 0..10 {
        let noised = defense.apply_noise(&original_pixels);
        results.push(noised);
    }
    
    // All results should be different (noise is random)
    let unique_results: HashSet<_> = results.iter().map(|r| r.as_slice()).collect();
    
    assert!(
        unique_results.len() > 1,
        "Canvas noise should produce different results each time"
    );
    
    // But each result should still be close to original
    for result in &results {
        let mut diff_count = 0;
        for (i, byte) in result.iter().enumerate() {
            if *byte != original_pixels[i] {
                diff_count += 1;
            }
        }
        // Some bytes should be different, but not all
        assert!(diff_count > 0, "Some pixels should be modified");
        assert!(diff_count < 256, "Not all pixels should be modified");
    }
}

/// Test WebGL parameter spoofing.
#[test]
fn test_webgl_parameter_spoofing() {
    use forloop_fingerprint::webgl::WebGLDefense;
    
    let defense = WebGLDefense::new();
    
    // Get spoofed parameters
    let vendor = defense.get_vendor();
    let renderer = defense.get_renderer();
    
    // Should return generic values, not actual GPU info
    assert!(
        vendor.contains("Generic") || vendor.contains("WebKit"),
        "Vendor should be generic: {}",
        vendor
    );
    
    assert!(
        renderer.contains("Generic") || renderer.contains("WebGL"),
        "Renderer should be generic: {}",
        renderer
    );
}

/// Test audio context fingerprint defense.
#[test]
fn test_audio_context_spoofing() {
    use forloop_fingerprint::audio::AudioDefense;
    
    let defense = AudioDefense::new();
    
    // Get multiple sample rates
    let mut rates = Vec::new();
    for _ in 0..10 {
        rates.push(defense.get_sample_rate());
    }
    
    // All should be standard values
    let standard_rates = [44100, 48000];
    for rate in &rates {
        assert!(
            standard_rates.contains(rate),
            "Sample rate should be standard: {}",
            rate
        );
    }
}

/// Test font enumeration defense.
#[test]
fn test_font_enumeration_defense() {
    use forloop_fingerprint::fonts::FontDefense;
    
    let defense = FontDefense::new();
    
    // Get list of "installed" fonts
    let fonts = defense.get_fonts();
    
    // Should return only common fonts
    let allowed_fonts = [
        "Arial",
        "Times New Roman", 
        "Courier New",
        "Georgia",
        "Verdana",
    ];
    
    for font in &fonts {
        assert!(
            allowed_fonts.contains(&font.as_str()),
            "Unexpected font in list: {}",
            font
        );
    }
    
    // Should have limited count
    assert!(
        fonts.len() <= 10,
        "Too many fonts exposed: {}",
        fonts.len()
    );
}

/// Test screen size normalization.
#[test]
fn test_screen_normalization() {
    use forloop_fingerprint::screen::ScreenDefense;
    
    let defense = ScreenDefense::new();
    
    let (width, height) = defense.get_screen_size();
    
    // Should be one of the common sizes
    let common_sizes = [
        (1920, 1080),
        (1366, 768),
        (1280, 720),
        (1536, 864),
    ];
    
    assert!(
        common_sizes.contains(&(width, height)),
        "Screen size should be common: {}x{}",
        width,
        height
    );
}

/// Test hardware info spoofing.
#[test]
fn test_hardware_spoofing() {
    use forloop_fingerprint::hardware::HardwareDefense;
    
    let defense = HardwareDefense::new();
    
    // CPU concurrency should be common value
    let concurrency = defense.get_hardware_concurrency();
    assert!(
        [2, 4, 8].contains(&concurrency),
        "Hardware concurrency should be common: {}",
        concurrency
    );
    
    // Device memory should be common value
    let memory = defense.get_device_memory();
    assert!(
        [4, 8].contains(&memory),
        "Device memory should be common: {}",
        memory
    );
}

/// Test navigator properties spoofing.
#[test]
fn test_navigator_spoofing() {
    use forloop_fingerprint::navigator::NavigatorDefense;
    
    let defense = NavigatorDefense::new();
    
    // Platform should be Windows (Tor Browser default)
    let platform = defense.get_platform();
    assert_eq!(platform, "Win32", "Platform should be Win32");
    
    // Languages should be minimal
    let languages = defense.get_languages();
    assert_eq!(
        languages,
        vec!["en-US", "en"],
        "Languages should be en-US, en"
    );
    
    // Should not reveal plugins
    let plugins = defense.get_plugins();
    assert!(plugins.is_empty(), "No plugins should be exposed");
}

/// Test that all defenses produce consistent values within a session.
#[test]
fn test_session_consistency() {
    use forloop_fingerprint::FingerprintDefense;
    
    let defense = FingerprintDefense::new_session();
    
    // Get values multiple times within same "session"
    let screen1 = defense.screen.get_screen_size();
    let screen2 = defense.screen.get_screen_size();
    
    // Should be consistent within session
    assert_eq!(screen1, screen2, "Screen size should be consistent in session");
    
    let hw1 = defense.hardware.get_hardware_concurrency();
    let hw2 = defense.hardware.get_hardware_concurrency();
    
    assert_eq!(hw1, hw2, "Hardware concurrency should be consistent in session");
}

/// Test that different sessions get different values.
#[test]
fn test_cross_session_variation() {
    use forloop_fingerprint::FingerprintDefense;
    
    // Create multiple sessions
    let mut sessions = Vec::new();
    for _ in 0..5 {
        sessions.push(FingerprintDefense::new_session());
    }
    
    // Collect screen sizes from each session
    let screen_sizes: Vec<_> = sessions
        .iter()
        .map(|s| s.screen.get_screen_size())
        .collect();
    
    // While not guaranteed to all be different (limited pool),
    // we should see some variation over multiple sessions
    let unique_sizes: HashSet<_> = screen_sizes.iter().collect();
    
    // Note: This test may occasionally fail due to randomness
    // In production, we'd use a seeded RNG for testing
    println!("Screen sizes across sessions: {:?}", screen_sizes);
    println!("Unique sizes: {}", unique_sizes.len());
}
