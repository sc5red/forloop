# forloop Threat Model

## Document Version
1.0.0 - Initial specification

## Core Guarantee

**Websites receive absolutely zero identifying or persistent data.**

This document defines what this means concretely, what attackers can do, what we defend against, and what we explicitly do not defend against.

---

## 1. Attacker Capabilities

### 1.1 Website Operator (Primary Threat)

**Capabilities:**
- Full control over served content (HTML, CSS, JS, images, etc.)
- Can execute arbitrary JavaScript within sandbox constraints
- Can set cookies, use Web Storage APIs, fingerprinting scripts
- Can measure timing with microsecond precision (if APIs available)
- Can correlate requests via HTTP headers, TLS fingerprints
- Can embed third-party resources
- Can attempt WebRTC leaks, DNS prefetch leaks
- Can use canvas, WebGL, AudioContext for fingerprinting
- Can enumerate fonts, plugins, screen properties
- Can measure scroll behavior, mouse movements, typing patterns

**forloop Defense:**
- All identifying APIs return synthetic, randomized, or null data
- No persistence mechanisms function
- Each request appears from different exit node
- Headers rotated per request
- TLS fingerprint matches large anonymity set

### 1.2 ISP / Network Observer

**Capabilities:**
- See all traffic metadata (timing, volume, destinations)
- Perform traffic correlation attacks
- Inject content on non-HTTPS connections
- Block or degrade specific protocols

**forloop Defense:**
- All traffic routed through multi-hop anonymized network
- Traffic padding and jitter applied
- Packet sizes normalized
- No plaintext HTTP ever (HTTPS-only mode)
- DNS over anonymized channel only

### 1.3 Global Passive Adversary

**Capabilities:**
- Observe traffic at multiple network points simultaneously
- Perform timing correlation across entry/exit points
- Statistical traffic analysis

**forloop Defense (Partial):**
- Traffic padding and delays
- Per-request circuit rotation
- NOTE: A truly global passive adversary with unlimited resources
  represents a physics-level limitation. We raise the cost, not eliminate the risk.

### 1.4 CDN / Third-Party Resource Provider

**Capabilities:**
- Track requests across multiple sites
- Fingerprint via resource timing
- Correlate cached resources

**forloop Defense:**
- RAM-only cache destroyed per request
- No cross-origin resource sharing
- Third-party requests appear from different circuits

### 1.5 Malicious Exit Node

**Capabilities:**
- See plaintext (HTTP) traffic
- Modify plaintext traffic
- Log destination IPs
- Perform SSL stripping attacks

**forloop Defense:**
- HTTPS-only mode (no HTTP)
- Strict certificate validation
- Exit node rotation per request (limits damage)
- Certificate pinning for critical domains

---

## 2. What Is Intentionally Broken

The following web features will not function. This is by design.

### 2.1 Completely Non-Functional

| Feature | Reason |
|---------|--------|
| Login sessions | No cookies, no storage |
| Shopping carts | No persistence |
| OAuth/SSO | Requires state |
| Payments | Requires identity |
| CAPTCHA (often) | Fingerprint-dependent |
| Two-factor auth | No device correlation |
| Push notifications | No service workers |
| Offline apps | No cache persistence |
| Web apps requiring storage | IndexedDB disabled |
| Real-time collaboration | Often fingerprints |
| Video conferencing | WebRTC disabled |
| File uploads requiring state | No session |
| Personalization | Intentionally prevented |

### 2.2 Degraded Functionality

| Feature | Limitation |
|---------|------------|
| Media playback | May have timing artifacts from fuzzed APIs |
| Games | Canvas/WebGL return synthetic data |
| Maps | Geolocation returns fake coords |
| Performance-critical apps | Timing APIs fuzzed |
| Large file downloads | Temp-only, opt-in |

### 2.3 User Expectations Broken

- No history - you cannot go back after closing tab
- No bookmarks - no persistence
- No saved passwords - no storage
- No "remember me" - by design
- Slow loading - anonymization overhead

---

## 3. Threats Out of Scope

### 3.1 Endpoint Compromise

If the user's machine is compromised:
- Keyloggers will capture input
- Screen capture will see content
- Memory forensics can extract data
- We assume a non-compromised endpoint

### 3.2 User Behavior Correlation

If the user:
- Logs into a personal account
- Enters personally identifying information
- Follows a unique browsing pattern
- Uses the same keywords repeatedly

**forloop cannot prevent this.** The browser prevents technical correlation, not behavioral correlation.

### 3.3 Physical Attacks

- Hardware keyloggers
- Shoulder surfing
- Evil maid attacks
- Cold boot attacks on RAM
- Out of scope

### 3.4 Targeted Malware

- Zero-day exploits targeting forloop specifically
- Nation-state attackers with forloop-specific capabilities
- We harden extensively but cannot guarantee against unknown zero-days

### 3.5 Rubber Hose Cryptanalysis

- Coercion attacks
- Legal compulsion
- Out of scope

---

## 4. Trust Boundaries

### 4.1 Trusted

- forloop binary (verified via signatures)
- Operating system kernel (assumed not malicious)
- Hardware (assumed not backdoored)
- Anonymization network (partially trusted, design assumes some nodes malicious)

### 4.2 Untrusted

- All websites
- All JavaScript
- All network infrastructure
- All CDNs
- All third parties
- Browser extensions (none allowed)
- User-installed fonts (not used)

---

## 5. Data Flow Guarantees

### 5.1 Outbound Data

| Data Type | Policy |
|-----------|--------|
| IP Address | Never exposed (multi-hop routing) |
| User-Agent | Synthetic, rotated per request |
| Accept-Language | Generic, rotated |
| Cookies | Blocked at engine level |
| Referer | Stripped or faked |
| Headers | Minimal, standardized set |
| TLS fingerprint | Matches Tor Browser fingerprint |
| Request timing | Jittered |
| DNS queries | Over anonymized channel only |

### 5.2 Inbound Data

| Data Type | Policy |
|-----------|--------|
| Cookies | Discarded |
| Set-Cookie | Ignored |
| Storage requests | Fail silently |
| Cache directives | Ignored (RAM-only) |
| Service Worker install | Blocked |
| Push subscription | Blocked |

### 5.3 Stored Data

| Data Type | Location | Lifetime |
|-----------|----------|----------|
| Page content | RAM only | Request duration |
| Rendered frames | RAM only | Tab duration |
| Downloads | Temp directory | Until explicit delete |
| Settings | Compiled-in defaults | N/A |
| History | None | N/A |
| Bookmarks | None | N/A |
| Passwords | None | N/A |

---

## 6. Fingerprinting Surface Analysis

### 6.1 Eliminated Vectors

| Vector | Defense |
|--------|---------|
| Canvas fingerprint | Deterministic fake output |
| WebGL fingerprint | Deterministic fake output |
| AudioContext fingerprint | Deterministic fake output |
| Font enumeration | Fixed virtual font set |
| Plugin enumeration | Empty list |
| Screen dimensions | Standardized buckets |
| Window dimensions | Standardized to screen |
| Device pixel ratio | Fixed value |
| CPU cores | Spoofed to common value |
| RAM | Spoofed to common value |
| Battery status | API disabled |
| Connection type | API disabled |
| Timezone | Fuzzed per request |
| Locale | Standardized |
| Do Not Track | Not sent (ironically identifies) |
| Media devices | Empty list |
| Gamepad | API disabled |
| WebRTC | Fully disabled |
| Bluetooth | API disabled |
| USB | API disabled |
| NFC | API disabled |
| Accelerometer | API disabled |
| Gyroscope | API disabled |
| Magnetometer | API disabled |
| Ambient light | API disabled |
| Keyboard layout | Standardized |
| Math precision | Standardized |

### 6.2 Timing-Based Vectors

| Vector | Defense |
|--------|---------|
| Date.now() | Reduced precision + jitter |
| performance.now() | Reduced to 100ms + jitter |
| requestAnimationFrame | Clamped to 60Hz + jitter |
| setTimeout/setInterval | Minimum 4ms, jittered |
| Resource timing | Disabled or fuzzed |
| Navigation timing | Fuzzed |
| SharedArrayBuffer | Disabled (Spectre) |

---

## 7. Validation Criteria

Before any release, verify:

1. **No request from the same session shares any identifying data**
2. **Requests from different tabs are unlinkable**
3. **Sequential requests to the same site appear from different users**
4. **All fingerprinting vectors return values from defined anonymity sets**
5. **No data persists between browser restarts**
6. **No data persists between request completions (for cache)**
7. **Network observer sees only encrypted traffic to anonymization entry nodes**
8. **Exit node sees requests but cannot correlate to user IP**

---

## 8. Residual Risks (Honest Assessment)

### 8.1 Behavioral Fingerprinting
Mouse movements, scroll patterns, typing rhythms can fingerprint.
**Mitigation:** Out of scope for v1. Future: input event normalization.

### 8.2 Traffic Analysis
Request sizes and timing patterns may be correlatable despite padding.
**Mitigation:** Padding and jitter reduce but don't eliminate.

### 8.3 Website-Specific Content
If user visits unique URL, content itself identifies.
**Mitigation:** User responsibility.

### 8.4 Zero-Day Exploits
Unknown browser vulnerabilities may leak data.
**Mitigation:** Aggressive sandboxing, minimal attack surface.

### 8.5 Anonymization Network Compromise
If majority of nodes controlled by adversary, deanonymization possible.
**Mitigation:** Use established network (Tor), consider multiple networks.

---

## 9. Security Assumptions

1. Cryptographic primitives (AES, ChaCha20, Ed25519, etc.) are secure
2. TLS 1.3 implementation is correct
3. Anonymization network protocol is sound
4. Compiler does not introduce backdoors (reproducible builds verify)
5. Random number generator is cryptographically secure
6. Memory isolation provided by OS works correctly
7. CPU does not have exploitable speculative execution bugs (partially mitigated)
