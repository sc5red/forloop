# Comparison: forloop vs. Existing Privacy Browsers

## Executive Summary

| Feature | forloop | Tor Browser | Mullvad Browser | Brave |
|---------|---------|-------------|-----------------|-------|
| **Network anonymity** | Per-request circuit | Session circuit | VPN (not Tor) | None by default |
| **IP hidden** | Always (Tor) | Always (Tor) | VPN required | Optional |
| **Fingerprinting** | Aggressively spoofed | Blends with crowd | Blends with crowd | Basic protection |
| **Storage** | Zero persistent | Ephemeral optional | Ephemeral optional | Normal |
| **Request correlation** | Impossible | Possible (same circuit) | Possible | Easy |
| **Cookie handling** | None stored | First-party ephemeral | First-party ephemeral | Third-party blocked |
| **Usability** | Sacrificed for privacy | Balanced | Balanced | Prioritized |

---

## Tor Browser

### What Tor Browser Does Right

1. **Tor network by default** - All traffic routed through Tor
2. **Fingerprinting resistance** - Uses Firefox with extensive patches
3. **Anonymity set** - Large user base = better blending
4. **Security slider** - Adjustable security levels
5. **Open source** - Auditable

### Where Tor Browser Falls Short

#### 1. Circuit Sharing

**Tor Browser:** Uses same circuit for all requests to a domain within a session.

**Problem:** If you visit `evil.com` and it embeds resources, those resources
can correlate with your other requests to the same exit node.

**forloop:** Each HTTP request gets a new circuit. No correlation possible.

#### 2. First-Party Isolation Incomplete

**Tor Browser:** Implements First-Party Isolation (FPI), but:
- Same origin still shares state within a session
- Closing tabs doesn't immediately clear state

**forloop:** Each request is completely isolated. There is no "session" concept
at the network level.

#### 3. JavaScript Fingerprinting Surface

**Tor Browser:** Tries to blend users by returning identical values.

**Problem:** If all Tor Browser users return `platform: "Win32"`, and you're on
Linux, your behavior may differ in detectable ways.

**forloop:** Actively spoofs with randomization per-session. You look different
each time, but never like yourself.

#### 4. Timing Attacks

**Tor Browser:** Uses reduced timer precision (100ms default).

**forloop:** Goes further:
- Artificial jitter on all requests
- Random padding on request/response bodies
- Traffic shaping to mask patterns

#### 5. WebGL/Canvas Fingerprinting

**Tor Browser:** Prompts before allowing canvas extraction (annoying UX).

**forloop:** Silently injects noise. No prompts. Fingerprinting just fails.

### forloop Advantage Summary

| Threat | Tor Browser | forloop |
|--------|-------------|---------|
| Request correlation | Possible within session | Impossible |
| Canvas fingerprinting | Prompt-based | Automatic noise |
| WebGL fingerprinting | Prompt-based | Spoofed/limited |
| Timing attacks | Reduced timers | Reduced + jitter |
| Traffic analysis | Basic padding | Active shaping |
| Circuit isolation | Per-domain | Per-request |

---

## Mullvad Browser

### What Mullvad Browser Does Right

1. **Based on Tor Browser** - Inherits fingerprinting resistance
2. **No Tor dependency** - Works with any VPN (or none)
3. **Clean UI** - Minimal branding
4. **Partnership with Tor Project** - Legitimate pedigree

### Where Mullvad Browser Falls Short

#### 1. No Network Anonymity Built-In

**Mullvad Browser:** Requires separate VPN (Mullvad VPN recommended).

**Problems:**
- VPN provider sees all your traffic
- Single point of trust (Mullvad)
- VPN doesn't provide anonymity, just changes IP
- If VPN disconnects, traffic leaks

**forloop:** Tor is integrated. No external trust required. Multiple relays
mean no single point sees origin AND destination.

#### 2. VPN vs. Tor

| Aspect | VPN (Mullvad) | Tor (forloop) |
|--------|---------------|---------------|
| Trust required | VPN provider | None (distributed) |
| Hops | 1 | 3+ |
| Logs | "No-log policy" (trust) | No central logs possible |
| Payment trail | Yes (even crypto) | None |
| Speed | Fast | Slower |
| Exit diversity | Fixed location | Random |

**forloop:** Does not require trusting any single entity.

#### 3. Same Session Correlation

**Mullvad Browser:** Like Tor Browser, allows request correlation within session.

**forloop:** No session-level correlation possible.

#### 4. Still Has Storage

**Mullvad Browser:** Allows ephemeral storage (optional).

**forloop:** No storage whatsoever. Cookies don't exist.

### forloop Advantage Summary

| Threat | Mullvad Browser | forloop |
|--------|-----------------|---------|
| VPN provider sees traffic | Yes | No VPN used |
| Network-level anonymity | VPN-dependent | Tor-native |
| Trust model | Single entity | Distributed |
| Payment tracking | VPN subscription | None |
| Request correlation | Possible | Impossible |

---

## Brave Browser

### What Brave Does Right

1. **Blocks ads/trackers** - Built-in ad blocking
2. **Fingerprinting protection** - Basic randomization
3. **Easy to use** - Drop-in Chrome replacement
4. **Tor window** - Optional Tor integration
5. **Mainstream adoption** - Large user base

### Where Brave Falls Completely Short

#### 1. Privacy is Optional

**Brave:** Privacy features must be enabled. Defaults are permissive.

**forloop:** Privacy is the only mode. There are no "normal" settings.

#### 2. Tor Integration is Weak

**Brave's Tor Windows:**
- Single circuit for all tabs (correlation possible)
- Fingerprinting protection not Tor-grade
- WebRTC may leak
- No traffic shaping
- Exit node reuse

**forloop:** Purpose-built Tor integration with per-request circuits.

#### 3. Fingerprinting "Protection" is Marketing

**Brave:** Randomizes some values slightly.

**Reality:**
- Randomization is predictable
- Many APIs not covered
- WebGL fully exposed
- Audio context exposed
- Canvas extraction allowed

**forloop:** Comprehensive defense across all APIs.

#### 4. Storage and Cookies

**Brave:** Blocks third-party cookies. Allows first-party.

**Problem:** First-party cookies still track you within a site.

**forloop:** No cookies. Period.

#### 5. Business Model

**Brave:** Makes money from crypto (BAT) and ads.

**Conflicts:**
- Brave Ads require knowing user interests
- Brave Rewards create identity
- Company has financial incentive to collect data

**forloop:** No business model. No company. No incentive to compromise.

#### 6. Chromium Base

**Brave:** Built on Chromium.

**Problems:**
- Inherits all Chromium fingerprinting surface
- Google can add fingerprinting vectors in engine updates
- Partitioning incomplete in Chromium vs. Firefox

**forloop:** Built on Firefox ESR (via LibreWolf).

### Brave's Failures in Detail

| Feature | Brave Claims | Reality |
|---------|--------------|---------|
| Fingerprinting protection | "Randomizes fingerprint" | Weak, detectable |
| Tor mode | "Private browsing with Tor" | No circuit isolation |
| Tracking protection | "Shields block trackers" | Allows first-party |
| No telemetry | "Doesn't collect data" | Sends usage stats (opt-out) |

### forloop Advantage Summary

| Threat | Brave | forloop |
|--------|-------|---------|
| IP exposure | Default exposed | Always Tor |
| Fingerprinting | Weak protection | Full defense |
| Cookie tracking | First-party allowed | None stored |
| Request correlation | Trivial | Impossible |
| Business model conflict | Yes (ads/crypto) | None |
| Chromium vulnerabilities | Inherited | Not applicable |

---

## Why forloop is Stronger

### 1. Per-Request Circuit Isolation

No other browser does this. Tor Browser uses per-domain circuits. Mullvad
uses whatever VPN provides. Brave's Tor mode uses a single circuit.

**forloop:** Every HTTP request = new circuit = no correlation.

### 2. Zero Storage

No other browser completely eliminates storage:
- Tor Browser: First-party ephemeral cookies allowed
- Mullvad: Same as Tor Browser
- Brave: Full storage by default

**forloop:** Nothing persists. Ever.

### 3. Aggressive Fingerprint Spoofing

Others try to blend or provide weak randomization.

**forloop:**
- Canvas: Noise injection
- WebGL: Parameter spoofing, WebGL 2.0 disabled
- Audio: Context spoofing
- Fonts: Curated allowlist
- Screen: Normalized to common sizes
- Hardware: Spoofed concurrency, memory, GPU
- Timing: Reduced resolution + jitter

### 4. Traffic Analysis Resistance

No other browser actively shapes traffic:

**forloop:**
- Request padding
- Response padding
- Random delays (jitter)
- Traffic pattern obfuscation

### 5. No Trust Required

| Browser | Trust Required |
|---------|---------------|
| Brave | Brave Software, Inc. |
| Mullvad | Mullvad VPN |
| Tor Browser | Tor Project (for updates) |
| forloop | Reproducible builds only |

forloop's reproducible builds mean you can verify the binary yourself.

### 6. Uncompromising Defaults

Others provide "options" and "settings" for privacy.

**forloop:** There is one mode. Maximum privacy. No options to weaken it.

---

## Threat Model Comparison

### Adversary: Website Tracking

| Defense | Tor Browser | Mullvad | Brave | forloop |
|---------|-------------|---------|-------|---------|
| IP hidden | ✓ | VPN only | ✗ | ✓ |
| Cookies blocked | Partial | Partial | Partial | ✓ |
| Fingerprint defense | Good | Good | Weak | Strong |
| Request correlation | Possible | Possible | Easy | Impossible |

### Adversary: ISP/Network

| Defense | Tor Browser | Mullvad | Brave | forloop |
|---------|-------------|---------|-------|---------|
| Traffic encrypted | ✓ | ✓ | ✗ | ✓ |
| Destination hidden | ✓ | VPN only | ✗ | ✓ |
| Tor usage hidden | ✗ | ✓ (VPN) | ✗ | ✗* |

*forloop doesn't hide Tor usage. Use bridges if needed.

### Adversary: Global Passive

| Defense | Tor Browser | Mullvad | Brave | forloop |
|---------|-------------|---------|-------|---------|
| Traffic shaping | ✗ | ✗ | ✗ | ✓ |
| Request padding | ✗ | ✗ | ✗ | ✓ |
| Circuit rotation | Per-domain | N/A | ✗ | Per-request |

---

## Conclusion

**Use Tor Browser if:** You want a balance of privacy and usability, with a
large anonymity set, and you trust the Tor Project.

**Use Mullvad Browser if:** You already use Mullvad VPN and want fingerprinting
protection without Tor.

**Use Brave if:** You want basic privacy with Chrome compatibility and don't
care about serious threats.

**Use forloop if:** You require absolute anonymity, accept that sites will
break, and understand that convenience is sacrificed for privacy.

---

## Summary Table

| Criterion | Tor Browser | Mullvad | Brave | forloop |
|-----------|-------------|---------|-------|---------|
| Privacy absolutism | 7/10 | 6/10 | 3/10 | 10/10 |
| Usability | 7/10 | 7/10 | 9/10 | 4/10 |
| Anonymity set | Large | Medium | Small | Self |
| Network anonymity | ✓ | VPN | ✗ | ✓ |
| Fingerprint defense | Good | Good | Weak | Aggressive |
| Storage elimination | Partial | Partial | None | Complete |
| Circuit isolation | Domain | N/A | None | Request |
| Traffic shaping | None | None | None | Active |
| Trust model | Tor Project | Mullvad AB | Brave Inc | None |
| Target user | Activist | Privacy-aware | Mainstream | Paranoid |
