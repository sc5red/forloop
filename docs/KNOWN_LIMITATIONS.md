# forloop Known Limitations

## Overview

forloop prioritizes privacy absolutism over convenience. This document
enumerates the fundamental limitations, tradeoffs, and compatibility issues
that result from this design philosophy.

---

## Physics-Level Limits

### Network Latency

**Problem:** Tor requires 3+ hops, adding 200-500ms latency per request.

**Impact:**
- Page loads are noticeably slower than clearnet
- Real-time applications (video chat, gaming) are unusable
- WebSocket connections experience high latency

**Why It Cannot Be Avoided:**
- Fewer hops would reduce anonymity
- Direct connections would reveal IP
- No amount of engineering can overcome speed-of-light delays across multiple continents

### Bandwidth Constraints

**Problem:** Tor network has finite capacity.

**Impact:**
- Large downloads are slow
- Streaming video is low quality or buffers
- Bulk operations (cloud backup, file sync) are impractical

**Why It Cannot Be Avoided:**
- Tor relays are volunteer-operated
- More bandwidth = more resources from community
- This is a social/economic problem, not technical

### Traffic Analysis

**Problem:** Global passive adversary can correlate traffic.

**Impact:**
- Nation-states with visibility into multiple network segments could
  potentially correlate entry/exit traffic
- Long-lived sessions are more vulnerable

**Why It Cannot Be Fully Avoided:**
- This is an active research problem
- Cover traffic helps but doesn't eliminate
- True solution requires protocol-level changes to internet

---

## Browser Compatibility Losses

### WebRTC Disabled

**What Breaks:**
- Google Meet, Zoom, Jitsi (in-browser)
- Discord voice chat
- Facebook Messenger video
- Any real-time audio/video

**Why:**
- WebRTC leaks local/public IP via STUN/TURN
- No way to proxy WebRTC through Tor safely
- Even "disabled" WebRTC can leak via ICE candidates

**Workaround:**
- Use dedicated apps (Signal Desktop, Zoom app) outside browser
- Those leak IP anyway, so use separate network

### WebGL Limited

**What Breaks:**
- Google Maps 3D view
- Some games
- Data visualization dashboards
- Three.js-heavy sites

**Why:**
- WebGL reveals GPU/driver fingerprint
- Extensions and parameters create unique signatures
- Shader performance is measurable

**Our Approach:**
- WebGL 1.0 allowed with spoofed parameters
- WebGL 2.0 disabled (too much fingerprinting surface)
- Some sites may render poorly or fall back to 2D

### Canvas Fingerprinting Defense

**What Breaks:**
- CAPTCHAs may trigger more often
- Some image editing sites
- Canvas-based games

**Why:**
- Canvas output is deterministic per-system
- Font rendering, antialiasing, color handling all leak info
- We add noise to prevent fingerprinting

**Impact:**
- Images have subtle noise (usually imperceptible)
- Consistent noise within session, different across sessions

### AudioContext Fingerprinting Defense

**What Breaks:**
- Some audio-based CAPTCHAs
- Web-based audio editors (Audacity alternatives)
- Music production apps

**Why:**
- AudioContext behavior is system-specific
- Sample rate, latency, processing all fingerprint

### Fonts Limited

**What Breaks:**
- Some sites with custom fonts may render differently
- Font enumeration scripts fail

**Why:**
- Installed fonts create unique fingerprint
- We expose only common fonts across platforms

### Storage Disabled

**What Breaks:**
- Saved preferences on sites
- Shopping carts (unless server-side)
- Logged-in sessions (must re-auth)
- Offline apps

**What We Block:**
- localStorage
- sessionStorage
- IndexedDB
- Cookies (first-party ephemeral only)
- Service Workers
- Cache API

**Workaround:**
- None. This is fundamental to the privacy model.
- Sites that require storage won't work as expected.

### JavaScript Timing Reduced

**What Breaks:**
- Some games (timing-sensitive)
- Performance monitoring
- Sites using high-resolution timing for legitimate purposes

**Why:**
- `performance.now()` resolution reduced to 100ms
- Date timing fuzzy
- Prevents Spectre-style attacks and timing fingerprinting

---

## Performance Tradeoffs

### Memory Usage

**Problem:** Multiple isolation mechanisms increase RAM usage.

**Impact:**
- Each tab has isolated process
- No shared caches between origins
- Expect 2-3x memory vs normal browser

**Why:**
- Isolation is the privacy model
- Sharing = potential leak channel

### CPU Usage

**Problem:** Fingerprinting defenses require computation.

**Impact:**
- Canvas operations slower (noise injection)
- Some cryptographic operations for Tor
- Traffic padding uses cycles

**Why:**
- Defense requires transformation
- Cannot be "free"

### Startup Time

**Problem:** Tor circuit establishment takes time.

**Impact:**
- 5-15 seconds to first page load
- Must wait for Tor bootstrap

**Why:**
- Building circuits requires consensus download
- Guard selection and negotiation takes time

### No Caching

**Problem:** Resources re-downloaded every session.

**Impact:**
- Same assets downloaded repeatedly
- No benefit from CDN caching (already neutralized)

**Why:**
- Cache is persistent storage
- Could be used to fingerprint or track

---

## Sites That Will Not Work

### Explicitly Incompatible

| Site Category | Reason | Status |
|--------------|--------|--------|
| Video conferencing | WebRTC required | Will not work |
| Some banking sites | Fingerprinting for "security" | May block |
| DRM video (Netflix 4K) | Widevine L1 requires hardware | Falls back to L3 |
| Some government sites | Aggressive bot detection | May block |
| Cloudflare challenges | Proof-of-work CAPTCHAs | Frequent |

### Degraded Experience

| Site Category | Issue | Workaround |
|--------------|-------|------------|
| Google services | Frequent CAPTCHAs | None (Google is hostile) |
| Social media | Can't stay logged in | Re-auth each session |
| E-commerce | Cart doesn't persist | Server-side carts work |
| News sites | Paywall resets | Actually a feature |

---

## Unsolvable Problems

### The "Logged In" Problem

If you log in to a site, you've identified yourself. forloop cannot help:
- The site knows your account
- They can track your activity within that account
- Logging in from Tor is still logging in

**Recommendation:** Create separate accounts for anonymous activity.

### The "Unique Behavior" Problem

If your behavior is unique, you're identifiable:
- Visiting an obscure combination of sites
- Typing patterns (if site captures)
- Mouse movement patterns (if site captures)

**forloop Cannot Fix:** Your behavior. Only you can blend in.

### The "Human Factor" Problem

Privacy fails when:
- You tell someone who you are
- You reuse identifiers (email, username)
- You access personal accounts
- You use unique phrases that can be searched

**forloop Cannot Fix:** OPSEC failures.

### The "Endpoint Security" Problem

forloop cannot protect against:
- Keyloggers on your system
- Screen capture malware
- Compromised hardware
- Shoulder surfing

**Assumption:** Your device is secure. If not, no browser helps.

---

## Comparison of Anonymity Levels

| Scenario | Anonymity Level | forloop Helps? |
|----------|----------------|----------------|
| ISP sees you visit a site | Yes | Tor hides destination |
| Site sees your IP | Yes | Tor hides IP |
| Site fingerprints browser | Yes | Defenses active |
| Site uses supercookies | Yes | No persistent storage |
| You log in to your account | No | You identified yourself |
| You write in unique style | No | Behavior-based ID |
| NSA with global visibility | Partial | Tor provides some cover |
| Physical access to device | No | Hardware level |

---

## Feature Requests We Will Never Implement

### Extensions

**Request:** "Add extension support for uBlock/NoScript/etc."

**Why No:**
- Extensions create unique fingerprint
- Each extension combination is identifiable
- Extension behavior can be detected
- We build protections in directly

### Cloud Sync

**Request:** "Sync my settings/bookmarks across devices."

**Why No:**
- Sync requires account = identity
- Sync server sees your data
- Ties sessions together across devices

### Password Manager Integration

**Request:** "Integrate with 1Password/Bitwarden."

**Why No:**
- Password manager knows all your accounts
- Creates identity link across sites
- Use separate password manager outside browser

### "Remember Me" Functionality

**Request:** "Stay logged in to sites."

**Why No:**
- Persistence = tracking
- Each session should be fresh
- This is fundamental to the model

---

## Performance Benchmarks

Typical impact (varies by hardware/network):

| Metric | Normal Browser | forloop | Overhead |
|--------|---------------|---------|----------|
| Page load (simple) | 0.5s | 2-5s | 4-10x |
| Page load (complex) | 2s | 5-15s | 2.5-7.5x |
| Memory per tab | 50-100MB | 100-200MB | 2x |
| Startup time | 1-2s | 10-20s | 10x |
| Video playback | 4K 60fps | 720p 30fps | Degraded |

---

## Acknowledgments

These limitations are not bugs. They are the necessary cost of:
- Routing through Tor
- Preventing fingerprinting
- Eliminating persistent state
- Isolating every request

**If a site breaks, that is acceptable.**

The goal is not to be a better Chrome. The goal is to be an anonymous browser
that happens to also render web pages. Privacy is the product. Convenience
is sacrificed.
