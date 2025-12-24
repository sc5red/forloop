//! forloop UI Components
//!
//! Minimal browser UI designed for privacy. No distractions, no tracking,
//! no unnecessary features. Every UI element serves a privacy purpose.

use std::sync::Arc;
use tokio::sync::mpsc;

/// Messages between UI and browser core.
#[derive(Debug, Clone)]
pub enum UiMessage {
    /// User typed in URL bar.
    Navigate(String),
    /// User clicked "New Loop" button.
    NewLoop,
    /// User clicked "Clear State" button.
    ClearState,
    /// Tor status changed.
    TorStatusChanged(TorStatus),
    /// Page load progress.
    LoadProgress(u8),
    /// Page title changed.
    TitleChanged(String),
    /// Security indicator changed.
    SecurityChanged(SecurityIndicator),
    /// Show error to user.
    ShowError(String),
    /// Exit browser.
    Quit,
}

/// Tor connection status.
#[derive(Debug, Clone, PartialEq)]
pub enum TorStatus {
    /// Not connected, trying to connect.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Connection failed.
    Failed(String),
    /// Building circuit.
    BuildingCircuit,
}

/// Security indicator state.
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityIndicator {
    /// HTTPS, onion service, or other secure connection.
    Secure,
    /// HTTP (insecure).
    Insecure,
    /// .onion address.
    Onion,
    /// Error state.
    Error,
}

/// Browser UI state.
pub struct BrowserUi {
    /// Current URL in the address bar.
    current_url: String,
    /// Current page title.
    current_title: String,
    /// Tor connection status.
    tor_status: TorStatus,
    /// Security indicator.
    security: SecurityIndicator,
    /// Page load progress (0-100).
    load_progress: u8,
    /// Channel to send messages to browser core.
    tx: mpsc::Sender<UiMessage>,
}

impl BrowserUi {
    /// Create new browser UI.
    pub fn new(tx: mpsc::Sender<UiMessage>) -> Self {
        Self {
            current_url: String::new(),
            current_title: String::from("forloop"),
            tor_status: TorStatus::Connecting,
            security: SecurityIndicator::Secure,
            load_progress: 0,
            tx,
        }
    }

    /// Handle incoming UI message.
    pub fn handle_message(&mut self, msg: UiMessage) {
        match msg {
            UiMessage::TorStatusChanged(status) => {
                self.tor_status = status;
            }
            UiMessage::LoadProgress(progress) => {
                self.load_progress = progress;
            }
            UiMessage::TitleChanged(title) => {
                self.current_title = title;
            }
            UiMessage::SecurityChanged(security) => {
                self.security = security;
            }
            _ => {}
        }
    }

    /// Navigate to a URL.
    pub async fn navigate(&mut self, url: &str) {
        self.current_url = url.to_string();
        self.load_progress = 0;
        let _ = self.tx.send(UiMessage::Navigate(url.to_string())).await;
    }

    /// Request new identity (new loop).
    pub async fn new_loop(&self) {
        let _ = self.tx.send(UiMessage::NewLoop).await;
    }

    /// Clear all state.
    pub async fn clear_state(&self) {
        let _ = self.tx.send(UiMessage::ClearState).await;
    }

    /// Get current Tor status for display.
    pub fn tor_status_display(&self) -> &'static str {
        match &self.tor_status {
            TorStatus::Connecting => "Connecting to Tor...",
            TorStatus::Connected => "Connected",
            TorStatus::Failed(_) => "Tor Failed",
            TorStatus::BuildingCircuit => "Building Circuit...",
        }
    }

    /// Get security indicator color.
    pub fn security_color(&self) -> &'static str {
        match self.security {
            SecurityIndicator::Secure => "#00ff00", // Green
            SecurityIndicator::Insecure => "#ff0000", // Red
            SecurityIndicator::Onion => "#7d4cdb", // Purple
            SecurityIndicator::Error => "#ffaa00", // Orange
        }
    }
}

/// Toolbar component.
pub struct Toolbar {
    /// Whether "New Loop" button is enabled.
    new_loop_enabled: bool,
}

impl Toolbar {
    /// Create new toolbar.
    pub fn new() -> Self {
        Self {
            new_loop_enabled: true,
        }
    }

    /// Render toolbar (pseudo-code for GTK/Qt integration).
    pub fn render(&self) -> ToolbarLayout {
        ToolbarLayout {
            items: vec![
                ToolbarItem::Button {
                    id: "new_loop",
                    label: "🔄 New Loop",
                    enabled: self.new_loop_enabled,
                    tooltip: "Create new identity (new Tor circuit, clear all state)",
                },
                ToolbarItem::Spacer,
                ToolbarItem::UrlBar {
                    id: "url_bar",
                    placeholder: "Enter .onion address or URL",
                },
                ToolbarItem::Spacer,
                ToolbarItem::TorIndicator {
                    id: "tor_status",
                },
                ToolbarItem::SecurityIndicator {
                    id: "security",
                },
            ],
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Toolbar layout for rendering.
pub struct ToolbarLayout {
    /// Items in the toolbar.
    pub items: Vec<ToolbarItem>,
}

/// Toolbar item types.
pub enum ToolbarItem {
    /// Button.
    Button {
        id: &'static str,
        label: &'static str,
        enabled: bool,
        tooltip: &'static str,
    },
    /// URL input bar.
    UrlBar {
        id: &'static str,
        placeholder: &'static str,
    },
    /// Tor status indicator.
    TorIndicator {
        id: &'static str,
    },
    /// Security indicator (lock icon).
    SecurityIndicator {
        id: &'static str,
    },
    /// Flexible spacer.
    Spacer,
}

/// Status bar at bottom of window.
pub struct StatusBar {
    /// Current status message.
    message: String,
    /// Current circuit info (anonymized).
    circuit_info: Option<CircuitInfo>,
}

/// Circuit information (displayed anonymously).
pub struct CircuitInfo {
    /// Country of exit node (ISO 3166-1 alpha-2).
    pub exit_country: String,
    /// Number of hops.
    pub hops: u8,
}

impl StatusBar {
    /// Create new status bar.
    pub fn new() -> Self {
        Self {
            message: String::new(),
            circuit_info: None,
        }
    }

    /// Set status message.
    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
    }

    /// Set circuit info.
    pub fn set_circuit(&mut self, info: CircuitInfo) {
        self.circuit_info = Some(info);
    }

    /// Get display text.
    pub fn display(&self) -> String {
        match &self.circuit_info {
            Some(circuit) => {
                format!(
                    "{} | Circuit: {} hops (exit: {})",
                    self.message, circuit.hops, circuit.exit_country
                )
            }
            None => self.message.clone(),
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Error dialog.
pub struct ErrorDialog {
    /// Error title.
    pub title: String,
    /// Error message.
    pub message: String,
    /// Whether to show "Report" button (always false for privacy).
    pub show_report: bool,
}

impl ErrorDialog {
    /// Create error dialog for connection failure.
    pub fn connection_failed(details: &str) -> Self {
        Self {
            title: String::from("Connection Failed"),
            message: format!(
                "Could not connect through Tor.\n\n\
                 This may be because:\n\
                 • Your network blocks Tor\n\
                 • The Tor network is experiencing issues\n\
                 • The destination is unreachable\n\n\
                 Technical details: {}",
                details
            ),
            show_report: false, // Never report - would leak info
        }
    }

    /// Create error dialog for certificate error.
    pub fn certificate_error(host: &str) -> Self {
        Self {
            title: String::from("Certificate Error"),
            message: format!(
                "The certificate for {} is invalid.\n\n\
                 forloop does not allow bypassing certificate errors.\n\
                 This protects you from man-in-the-middle attacks.",
                host
            ),
            show_report: false,
        }
    }
}

/// Onboarding screen shown on first run.
pub struct OnboardingScreen {
    /// Current page index.
    current_page: usize,
}

impl OnboardingScreen {
    /// Create new onboarding screen.
    pub fn new() -> Self {
        Self { current_page: 0 }
    }

    /// Get current page content.
    pub fn current_content(&self) -> OnboardingPage {
        let pages = Self::pages();
        pages[self.current_page].clone()
    }

    /// Go to next page.
    pub fn next(&mut self) -> bool {
        if self.current_page < Self::pages().len() - 1 {
            self.current_page += 1;
            true
        } else {
            false
        }
    }

    /// Define onboarding pages.
    fn pages() -> Vec<OnboardingPage> {
        vec![
            OnboardingPage {
                title: String::from("Welcome to forloop"),
                content: String::from(
                    "forloop is a browser designed for one thing:\n\
                     absolute anonymity.\n\n\
                     Every website sees a different you.\n\
                     No fingerprints. No tracking. No history.",
                ),
                icon: "🔒",
            },
            OnboardingPage {
                title: String::from("How It Works"),
                content: String::from(
                    "All traffic goes through Tor.\n\
                     Each request uses a new circuit.\n\
                     Nothing is stored between sessions.\n\n\
                     Sites will load slower. Some will break.\n\
                     This is the cost of true privacy.",
                ),
                icon: "🧅",
            },
            OnboardingPage {
                title: String::from("What forloop Cannot Do"),
                content: String::from(
                    "If you log in to a site, you identify yourself.\n\
                     If your behavior is unique, you're trackable.\n\
                     If your device is compromised, nothing helps.\n\n\
                     forloop protects the browser. You protect yourself.",
                ),
                icon: "⚠️",
            },
            OnboardingPage {
                title: String::from("Ready"),
                content: String::from(
                    "Click 'New Loop' anytime to get a new identity.\n\n\
                     Remember: privacy requires discipline.\n\
                     forloop gives you the tools.\n\
                     You must use them wisely.",
                ),
                icon: "✓",
            },
        ]
    }
}

impl Default for OnboardingScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Onboarding page content.
#[derive(Clone)]
pub struct OnboardingPage {
    /// Page title.
    pub title: String,
    /// Page content.
    pub content: String,
    /// Icon emoji.
    pub icon: &'static str,
}

/// Settings panel (very minimal).
pub struct SettingsPanel {
    /// Current settings values.
    settings: SettingsValues,
}

/// Settings values (very few options by design).
#[derive(Clone)]
pub struct SettingsValues {
    /// Use bridges (for censored networks).
    pub use_bridges: bool,
    /// Bridge lines (if use_bridges is true).
    pub bridge_lines: Vec<String>,
    /// Security level (always maximum, not changeable).
    pub security_level: SecurityLevel,
}

/// Security level.
#[derive(Clone, PartialEq)]
pub enum SecurityLevel {
    /// Maximum security (the only option).
    Maximum,
}

impl SettingsPanel {
    /// Create new settings panel.
    pub fn new() -> Self {
        Self {
            settings: SettingsValues {
                use_bridges: false,
                bridge_lines: vec![],
                security_level: SecurityLevel::Maximum,
            },
        }
    }

    /// Get available settings.
    pub fn available_settings(&self) -> Vec<SettingItem> {
        vec![
            SettingItem::Toggle {
                id: "use_bridges",
                label: "Use Tor Bridges",
                description: "Connect through bridges to bypass censorship",
                value: self.settings.use_bridges,
            },
            SettingItem::TextArea {
                id: "bridge_lines",
                label: "Bridge Lines",
                description: "One bridge per line (get bridges at bridges.torproject.org)",
                value: self.settings.bridge_lines.join("\n"),
                visible_when: "use_bridges",
            },
            SettingItem::Info {
                label: "Security Level",
                value: "Maximum (cannot be changed)",
                description: "forloop always operates at maximum security",
            },
        ]
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Setting item types.
pub enum SettingItem {
    /// Boolean toggle.
    Toggle {
        id: &'static str,
        label: &'static str,
        description: &'static str,
        value: bool,
    },
    /// Multi-line text input.
    TextArea {
        id: &'static str,
        label: &'static str,
        description: &'static str,
        value: String,
        visible_when: &'static str,
    },
    /// Read-only info.
    Info {
        label: &'static str,
        value: &'static str,
        description: &'static str,
    },
}

/// Window manager integration.
pub struct WindowManager {
    /// Window title.
    title: String,
    /// Window dimensions.
    width: u32,
    height: u32,
}

impl WindowManager {
    /// Create new window with standard dimensions.
    pub fn new() -> Self {
        // Use common window size to avoid fingerprinting
        Self {
            title: String::from("forloop"),
            width: 1280,
            height: 720,
        }
    }

    /// Get window title (never includes URL for privacy).
    pub fn title(&self) -> &str {
        // Always "forloop" - never show URL in title
        // (could be captured by screen sharing, window lists, etc.)
        &self.title
    }

    /// Get recommended window size.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tor_status_display() {
        let (tx, _rx) = mpsc::channel(10);
        let ui = BrowserUi::new(tx);

        assert_eq!(ui.tor_status_display(), "Connecting to Tor...");
    }

    #[test]
    fn test_security_color() {
        let (tx, _rx) = mpsc::channel(10);
        let ui = BrowserUi::new(tx);

        assert_eq!(ui.security_color(), "#00ff00");
    }

    #[test]
    fn test_window_title_never_shows_url() {
        let wm = WindowManager::new();
        assert_eq!(wm.title(), "forloop");
    }

    #[test]
    fn test_settings_security_not_changeable() {
        let panel = SettingsPanel::new();
        assert_eq!(panel.settings.security_level, SecurityLevel::Maximum);
    }
}
