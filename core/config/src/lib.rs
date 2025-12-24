//! forloop CLI and Configuration
//!
//! This module handles command-line arguments and secure-by-default configuration.
//! There are intentionally NO options to weaken privacy guarantees.

use std::path::PathBuf;

/// forloop command-line interface.
#[derive(Debug)]
pub struct ForloopCli {
    /// URL to open (optional)
    pub url: Option<String>,
    /// Start with completely fresh state
    pub new_loop: bool,
    /// Kill all state and exit
    pub kill_all_state: bool,
    /// Use bridges for Tor
    pub use_bridges: bool,
    /// Custom bridge lines
    pub bridges: Vec<String>,
    /// Verbose logging (to stderr only)
    pub verbose: bool,
    /// Print version and exit
    pub version: bool,
    /// Print help and exit
    pub help: bool,
}

impl ForloopCli {
    /// Parse command-line arguments.
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::parse_args(&args)
    }

    fn parse_args(args: &[String]) -> Self {
        let mut cli = Self {
            url: None,
            new_loop: false,
            kill_all_state: false,
            use_bridges: false,
            bridges: Vec::new(),
            verbose: false,
            version: false,
            help: false,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--new-loop" | "-n" => {
                    cli.new_loop = true;
                }
                "--kill-all-state" | "-k" => {
                    cli.kill_all_state = true;
                }
                "--use-bridges" => {
                    cli.use_bridges = true;
                }
                "--bridge" => {
                    i += 1;
                    if i < args.len() {
                        cli.bridges.push(args[i].clone());
                    }
                }
                "--verbose" | "-v" => {
                    cli.verbose = true;
                }
                "--version" | "-V" => {
                    cli.version = true;
                }
                "--help" | "-h" => {
                    cli.help = true;
                }
                arg if !arg.starts_with('-') => {
                    // Assume it's a URL
                    cli.url = Some(arg.to_string());
                }
                _ => {
                    // Unknown option - ignore for forward compatibility
                }
            }
            i += 1;
        }

        cli
    }

    /// Print help message.
    pub fn print_help() {
        println!(
            r#"forloop - Every request is the first

USAGE:
    forloop [OPTIONS] [URL]

ARGUMENTS:
    [URL]    URL to open on startup (optional)

OPTIONS:
    -n, --new-loop          Start with completely fresh state (always true)
    -k, --kill-all-state    Securely wipe all temporary data and exit
        --use-bridges       Use Tor bridges for censorship circumvention
        --bridge <BRIDGE>   Specify a bridge line (can be repeated)
    -v, --verbose           Enable verbose logging to stderr
    -V, --version           Print version information
    -h, --help              Print this help message

NOTES:
    forloop has no persistent state. Every session starts fresh.
    There are no options to weaken privacy guarantees.
    All connections go through Tor. This is not configurable.

EXAMPLES:
    forloop                         Start with blank page
    forloop https://example.onion   Open a specific URL
    forloop --kill-all-state        Wipe temp data and exit
    forloop --use-bridges           Use bridges in censored regions

PHILOSOPHY:
    Stateless by design.
    Memory is a vulnerability.
    Every request is the first.
"#
        );
    }

    /// Print version.
    pub fn print_version() {
        println!("forloop {}", env!("CARGO_PKG_VERSION"));
        println!("Engine: Gecko (Firefox ESR 128)");
        println!("Tor: Embedded");
        println!();
        println!("Motto: Every request is the first.");
    }
}

/// Secure-by-default configuration.
/// These values are compiled in and CANNOT be changed at runtime.
#[derive(Debug, Clone)]
pub struct ForloopConfig {
    // Network settings
    /// Tor SOCKS port
    pub tor_socks_port: u16,
    /// Tor control port
    pub tor_control_port: u16,
    /// Create new circuit per request
    pub new_circuit_per_request: bool,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    // Fingerprint settings
    /// Timing precision in milliseconds
    pub timing_precision_ms: u64,
    /// Screen size bucket to use
    pub screen_bucket: ScreenBucket,

    // Storage settings (all disabled)
    /// Cookies enabled (always false)
    pub cookies_enabled: bool,
    /// Local storage enabled (always false)
    pub local_storage_enabled: bool,
    /// Session storage enabled (always false)
    pub session_storage_enabled: bool,
    /// IndexedDB enabled (always false)
    pub indexed_db_enabled: bool,
    /// Cache enabled (always false for disk)
    pub disk_cache_enabled: bool,
    /// Service workers enabled (always false)
    pub service_workers_enabled: bool,

    // Security settings
    /// WebRTC enabled (always false)
    pub webrtc_enabled: bool,
    /// Geolocation enabled (always false)
    pub geolocation_enabled: bool,
    /// Sensors enabled (always false)
    pub sensors_enabled: bool,

    // Telemetry settings (all disabled)
    /// Telemetry enabled (always false)
    pub telemetry_enabled: bool,
    /// Crash reporter enabled (always false)
    pub crash_reporter_enabled: bool,
}

/// Screen size bucket for fingerprint defense.
#[derive(Debug, Clone, Copy)]
pub struct ScreenBucket {
    pub width: u32,
    pub height: u32,
}

impl Default for ForloopConfig {
    fn default() -> Self {
        Self {
            // Network
            tor_socks_port: 9150,
            tor_control_port: 9151,
            new_circuit_per_request: true,
            request_timeout_secs: 60,

            // Fingerprint
            timing_precision_ms: 100,
            screen_bucket: ScreenBucket {
                width: 1920,
                height: 1080,
            },

            // Storage - ALL DISABLED
            cookies_enabled: false,
            local_storage_enabled: false,
            session_storage_enabled: false,
            indexed_db_enabled: false,
            disk_cache_enabled: false,
            service_workers_enabled: false,

            // Security - MAXIMUM
            webrtc_enabled: false,
            geolocation_enabled: false,
            sensors_enabled: false,

            // Telemetry - ALL DISABLED
            telemetry_enabled: false,
            crash_reporter_enabled: false,
        }
    }
}

impl ForloopConfig {
    /// Get the singleton configuration.
    /// This returns compiled-in defaults that cannot be modified.
    pub fn get() -> &'static Self {
        static CONFIG: ForloopConfig = ForloopConfig {
            tor_socks_port: 9150,
            tor_control_port: 9151,
            new_circuit_per_request: true,
            request_timeout_secs: 60,
            timing_precision_ms: 100,
            screen_bucket: ScreenBucket {
                width: 1920,
                height: 1080,
            },
            cookies_enabled: false,
            local_storage_enabled: false,
            session_storage_enabled: false,
            indexed_db_enabled: false,
            disk_cache_enabled: false,
            service_workers_enabled: false,
            webrtc_enabled: false,
            geolocation_enabled: false,
            sensors_enabled: false,
            telemetry_enabled: false,
            crash_reporter_enabled: false,
        };

        &CONFIG
    }

    /// Verify configuration is secure.
    /// Panics if any privacy-weakening options are enabled.
    pub fn verify_secure(&self) {
        assert!(!self.cookies_enabled, "Cookies must be disabled");
        assert!(!self.local_storage_enabled, "Local storage must be disabled");
        assert!(
            !self.session_storage_enabled,
            "Session storage must be disabled"
        );
        assert!(!self.indexed_db_enabled, "IndexedDB must be disabled");
        assert!(!self.disk_cache_enabled, "Disk cache must be disabled");
        assert!(
            !self.service_workers_enabled,
            "Service workers must be disabled"
        );
        assert!(!self.webrtc_enabled, "WebRTC must be disabled");
        assert!(!self.geolocation_enabled, "Geolocation must be disabled");
        assert!(!self.sensors_enabled, "Sensors must be disabled");
        assert!(!self.telemetry_enabled, "Telemetry must be disabled");
        assert!(
            !self.crash_reporter_enabled,
            "Crash reporter must be disabled"
        );
        assert!(
            self.new_circuit_per_request,
            "New circuit per request must be enabled"
        );
    }
}

/// Temporary directory for downloads (RAM-backed).
pub fn get_temp_download_dir() -> PathBuf {
    // Use RAM-backed tmpfs on Linux
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/dev/shm/forloop-downloads")
    }

    #[cfg(not(target_os = "linux"))]
    {
        std::env::temp_dir().join("forloop-downloads")
    }
}

/// Securely wipe all temporary data.
pub fn kill_all_state() -> std::io::Result<()> {
    let temp_dir = get_temp_download_dir();

    if temp_dir.exists() {
        // Overwrite files before deleting
        secure_delete_dir(&temp_dir)?;
    }

    // Clear any other temporary state
    // (In a full implementation, this would wipe Tor state, etc.)

    Ok(())
}

/// Securely delete a directory by overwriting files first.
fn secure_delete_dir(path: &PathBuf) -> std::io::Result<()> {
    use std::fs;
    use std::io::Write;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            secure_delete_dir(&path)?;
        } else {
            // Overwrite file with zeros
            let len = fs::metadata(&path)?.len();
            let mut file = fs::OpenOptions::new().write(true).open(&path)?;

            let zeros = vec![0u8; 4096];
            let mut remaining = len as usize;

            while remaining > 0 {
                let to_write = remaining.min(zeros.len());
                file.write_all(&zeros[..to_write])?;
                remaining -= to_write;
            }

            file.sync_all()?;
            drop(file);

            // Now delete
            fs::remove_file(&path)?;
        }
    }

    fs::remove_dir(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = vec![
            "forloop".to_string(),
            "--new-loop".to_string(),
            "https://example.onion".to_string(),
        ];

        let cli = ForloopCli::parse_args(&args);
        assert!(cli.new_loop);
        assert_eq!(cli.url, Some("https://example.onion".to_string()));
    }

    #[test]
    fn test_cli_bridges() {
        let args = vec![
            "forloop".to_string(),
            "--use-bridges".to_string(),
            "--bridge".to_string(),
            "obfs4 192.168.1.1:443".to_string(),
        ];

        let cli = ForloopCli::parse_args(&args);
        assert!(cli.use_bridges);
        assert_eq!(cli.bridges.len(), 1);
    }

    #[test]
    fn test_config_defaults() {
        let config = ForloopConfig::default();

        // All privacy-weakening features must be disabled
        assert!(!config.cookies_enabled);
        assert!(!config.local_storage_enabled);
        assert!(!config.webrtc_enabled);
        assert!(!config.telemetry_enabled);
        assert!(config.new_circuit_per_request);
    }

    #[test]
    fn test_config_verification() {
        let config = ForloopConfig::default();
        config.verify_secure(); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Cookies must be disabled")]
    fn test_config_verification_fails_on_cookies() {
        let mut config = ForloopConfig::default();
        config.cookies_enabled = true;
        config.verify_secure();
    }
}
