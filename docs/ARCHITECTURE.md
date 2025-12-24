# forloop Architecture

## Engine Choice: Firefox (Gecko) via LibreWolf Fork

### Why Not Chromium

1. **Google's fingerprinting-friendly design**: Chromium exposes many APIs by default that require patching
2. **Blink's complexity**: More attack surface, harder to audit
3. **Update velocity**: Chromium updates frequently, making patch maintenance costly
4. **WebRTC integration**: Deeply integrated, harder to fully disable
5. **Telemetry hooks**: Pervasive throughout codebase

### Why Not Custom Engine

1. **Decades of security work**: Would be discarded
2. **Web compatibility**: No custom engine achieves parity
3. **Audit cost**: Impossible for small team
4. **Time to production**: Years, not months

### Why Firefox/Gecko

1. **Proven privacy fork**: LibreWolf and Tor Browser both use Gecko
2. **about:config**: Extensive runtime configuration
3. **WebExtension isolation**: Better than Chromium's
4. **Servo components**: Rust-based safety in critical paths
5. **Established patch sets**: Tor Browser patches as reference
6. **Mozilla's privacy stance**: Less hostile baseline

### Specific Base

- **Engine**: Gecko (Firefox ESR 128.x branch)
- **Fork base**: LibreWolf (as starting point for de-googled build)
- **Reference patches**: Tor Browser patches (selectively applied)
- **Version pinning**: ESR for stability, security backports

### Tradeoffs Acknowledged

- Firefox market share declining (fingerprint diversity concern)
- Some sites "optimize" for Chrome only (acceptable breakage)
- JIT security concerns (mitigated by disabling JIT)

---

## Full Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              FORLOOP BROWSER                                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────────────┐ │
│  │                           UI PROCESS (Privileged)                            │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   Minimal   │  │   Address   │  │    Tab      │  │   Security          │ │ │
│  │  │   Chrome    │  │     Bar     │  │   Strip     │  │   Indicators        │ │ │
│  │  │   (GTK4)    │  │  (Display   │  │  (No state) │  │   (Onion, HTTPS)    │ │ │
│  │  │             │  │    Only)    │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  └────────────────────────────────────────┬────────────────────────────────────┘ │
│                                           │ IPC (Unix Domain Socket)             │
│  ┌────────────────────────────────────────▼────────────────────────────────────┐ │
│  │                        BROKER PROCESS (Coordinator)                          │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │ │
│  │  │  Request        │  │  Header         │  │  Fingerprint               │  │ │
│  │  │  Orchestrator   │  │  Synthesizer    │  │  Defense Controller        │  │ │
│  │  │                 │  │                 │  │                            │  │ │
│  │  │  - Routes req   │  │  - UA rotation  │  │  - Coordinates all         │  │ │
│  │  │  - Enforces     │  │  - Accept-Lang  │  │    defense modules         │  │ │
│  │  │    policy       │  │  - No Referer   │  │  - Generates per-request   │  │ │
│  │  │  - No cache     │  │  - Minimal set  │  │    synthetic identity      │  │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────────────┘  │ │
│  └────────────────────────────────────────┬────────────────────────────────────┘ │
│                                           │ IPC                                  │
│  ┌────────────────────────────────────────▼────────────────────────────────────┐ │
│  │                         NETWORK PROCESS (Isolated)                           │ │
│  │  ┌─────────────────────────────────────────────────────────────────────────┐│ │
│  │  │                        ANONYMIZATION LAYER                               ││ │
│  │  │  ┌───────────────┐  ┌───────────────┐  ┌──────────────────────────────┐ ││ │
│  │  │  │  Circuit      │  │  Traffic      │  │  TLS Fingerprint            │ ││ │
│  │  │  │  Manager      │  │  Shaper       │  │  Normalizer                 │ ││ │
│  │  │  │               │  │               │  │                             │ ││ │
│  │  │  │  - Per-req    │  │  - Padding    │  │  - Matches Tor Browser      │ ││ │
│  │  │  │    rotation   │  │  - Jitter     │  │  - Consistent cipher order  │ ││ │
│  │  │  │  - Multi-hop  │  │  - Delay      │  │  - Standard extensions      │ ││ │
│  │  │  │  - Path sel   │  │  - Normalize  │  │                             │ ││ │
│  │  │  └───────────────┘  └───────────────┘  └──────────────────────────────┘ ││ │
│  │  └─────────────────────────────────────────────────────────────────────────┘│ │
│  │  ┌─────────────────────────────────────────────────────────────────────────┐│ │
│  │  │                         TOR INTEGRATION                                  ││ │
│  │  │  ┌───────────────┐  ┌───────────────┐  ┌──────────────────────────────┐ ││ │
│  │  │  │  Embedded     │  │  SOCKS5       │  │  DNS over Tor               │ ││ │
│  │  │  │  tor daemon   │  │  Handler      │  │  (No system DNS)            │ ││ │
│  │  │  │  (arti/c-tor) │  │               │  │                             │ ││ │
│  │  │  └───────────────┘  └───────────────┘  └──────────────────────────────┘ ││ │
│  │  └─────────────────────────────────────────────────────────────────────────┘│ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                           │ IPC                                  │
│  ┌────────────────────────────────────────▼────────────────────────────────────┐ │
│  │                    CONTENT PROCESS(ES) (Sandboxed)                           │ │
│  │  ┌─────────────────────────────────────────────────────────────────────────┐│ │
│  │  │                         GECKO RENDERER                                   ││ │
│  │  │  ┌───────────────────────────────────────────────────────────────────┐  ││ │
│  │  │  │                    JS ENGINE (SpiderMonkey)                        │  ││ │
│  │  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐ │  ││ │
│  │  │  │  │  JIT OFF    │  │  API Shims  │  │  Timing     │  │ No Shared │ │  ││ │
│  │  │  │  │  (Interp    │  │  (Canvas,   │  │  Fuzzer     │  │ Memory    │ │  ││ │
│  │  │  │  │   only)     │  │  WebGL,     │  │  (Date,     │  │ (SAB off) │ │  ││ │
│  │  │  │  │             │  │  Audio)     │  │  perf.now)  │  │           │ │  ││ │
│  │  │  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘ │  ││ │
│  │  │  └───────────────────────────────────────────────────────────────────┘  ││ │
│  │  │  ┌───────────────────────────────────────────────────────────────────┐  ││ │
│  │  │  │                  FINGERPRINT DEFENSE LAYER                         │  ││ │
│  │  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐ │  ││ │
│  │  │  │  │  Canvas     │  │  Font       │  │  Screen     │  │ Hardware  │ │  ││ │
│  │  │  │  │  Spoofer    │  │  Virtualizer│  │  Normalizer │  │ Spoofer   │ │  ││ │
│  │  │  │  │  (Fixed     │  │  (12 fonts  │  │  (Bucket    │  │ (CPU:4,   │ │  ││ │
│  │  │  │  │   output)   │  │   only)     │  │   sizes)    │  │  RAM:8GB) │ │  ││ │
│  │  │  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘ │  ││ │
│  │  │  └───────────────────────────────────────────────────────────────────┘  ││ │
│  │  │  ┌───────────────────────────────────────────────────────────────────┐  ││ │
│  │  │  │                    STORAGE BLOCKERS                                │  ││ │
│  │  │  │  • Cookies: intercepted, discarded                                 │  ││ │
│  │  │  │  • localStorage: returns null                                      │  ││ │
│  │  │  │  • sessionStorage: returns null                                    │  ││ │
│  │  │  │  • IndexedDB: throws SecurityError                                 │  ││ │
│  │  │  │  • Cache API: no-op                                                │  ││ │
│  │  │  │  • Service Workers: registration fails                             │  ││ │
│  │  │  └───────────────────────────────────────────────────────────────────┘  ││ │
│  │  └─────────────────────────────────────────────────────────────────────────┘│ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

                              EXTERNAL DEPENDENCIES

┌─────────────────────────────────────────────────────────────────────────────────┐
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────────┐  │
│  │  TOR NETWORK    │  │  SYSTEM         │  │  TEMP FILESYSTEM                │  │
│  │  (External)     │  │  (Isolated)     │  │  (RAM-backed tmpfs)             │  │
│  │                 │  │                 │  │                                 │  │
│  │  - Entry nodes  │  │  - No DNS calls │  │  - Downloads only               │  │
│  │  - Middle nodes │  │  - No telemetry │  │  - Wiped on exit                │  │
│  │  - Exit nodes   │  │  - No autoconf  │  │  - Encrypted                    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Process Model

### Process Types and Privileges

| Process | Privilege Level | Capabilities | Sandbox |
|---------|-----------------|--------------|---------|
| UI | High (user-level) | Display, input handling | None (minimal code) |
| Broker | Medium | IPC routing, policy enforcement | seccomp-bpf |
| Network | Low | Network I/O only | seccomp-bpf, no filesystem |
| Content | Minimal | Render, JS execution | seccomp-bpf, namespaces, no network |

### IPC Design

All IPC uses Unix domain sockets with:
- Message authentication (HMAC)
- Length-prefix framing
- Strict message type validation
- No shared memory (prevents Spectre-class attacks)

```
UI Process ◄─────► Broker Process ◄─────► Network Process
                        │
                        ▼
                Content Process(es)
```

### Privilege Separation

```
┌──────────────────────────────────────────────────────────────┐
│                    PRIVILEGE LEVELS                           │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Level 3 (Highest)     ┌─────────────────────────────────┐   │
│  UI Process            │  • Can spawn other processes     │   │
│                        │  • Handles user input            │   │
│                        │  • Minimal attack surface        │   │
│                        └─────────────────────────────────┘   │
│                                     │                         │
│                                     ▼                         │
│  Level 2               ┌─────────────────────────────────┐   │
│  Broker Process        │  • Enforces all policies         │   │
│                        │  • Routes messages               │   │
│                        │  • Generates synthetic data      │   │
│                        │  • No direct network access      │   │
│                        └─────────────────────────────────┘   │
│                                     │                         │
│                          ┌──────────┴──────────┐              │
│                          ▼                     ▼              │
│  Level 1     ┌───────────────────┐  ┌───────────────────┐    │
│              │  Network Process  │  │  Content Process  │    │
│              │  • Network I/O    │  │  • Rendering only │    │
│              │  • No filesystem  │  │  • No network     │    │
│              │  • No IPC except  │  │  • No filesystem  │    │
│              │    to Broker      │  │  • Heavy sandbox  │    │
│              └───────────────────┘  └───────────────────┘    │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

---

## Component Details

### 1. Header Synthesizer

Located in Broker Process. Generates HTTP headers per-request:

- **User-Agent**: Rotates from curated list matching Tor Browser fingerprint
- **Accept-Language**: `en-US,en;q=0.5` (generic)
- **Accept**: Standard values
- **Referer**: Never sent
- **DNT**: Not sent (identifies users)
- **Custom headers**: Stripped

### 2. Fingerprint Defense Controller

Coordinates all fingerprinting defenses:

- Generates per-request "synthetic identity"
- Distributes to Content Process
- Ensures consistency within request
- Rotates between requests

### 3. Circuit Manager

In Network Process:

- Maintains connection to Tor network
- Creates new circuit per request
- Implements circuit isolation policy
- Handles circuit failures gracefully

### 4. Traffic Shaper

- Adds random padding to requests/responses
- Introduces jitter (0-50ms)
- Normalizes packet sizes
- Delays to prevent timing correlation

### 5. JS API Shims

Injected into every page:

- `Date.now()`: Reduced precision + jitter
- `performance.now()`: 100ms precision
- `navigator.*`: Synthetic values
- `screen.*`: Bucketed values
- `canvas.*`: Deterministic output
- `WebGL*`: Deterministic output

### 6. Storage Blockers

At engine level (C++ patches):

- Cookie jar: Permanently empty
- Web Storage: Returns null
- IndexedDB: Disabled
- Cache: RAM-only, per-request

---

## Memory Architecture

### RAM-Only Policy

```
┌─────────────────────────────────────────────────────────────┐
│                     MEMORY LAYOUT                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              CONTENT PROCESS MEMORY                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐  │   │
│  │  │  DOM Tree   │  │  Render     │  │  JS Heap     │  │   │
│  │  │  (Temp)     │  │  Tree       │  │  (No persist)│  │   │
│  │  │             │  │  (Temp)     │  │              │  │   │
│  │  └─────────────┘  └─────────────┘  └──────────────┘  │   │
│  │                                                       │   │
│  │  ┌─────────────────────────────────────────────────┐ │   │
│  │  │              CACHE (RAM-Only)                    │ │   │
│  │  │  • Destroyed after each request                  │ │   │
│  │  │  • Never written to disk                         │ │   │
│  │  │  • Keyed by request, not URL                     │ │   │
│  │  └─────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                   FORBIDDEN                           │   │
│  │  ✗ Disk cache                                         │   │
│  │  ✗ Cookie files                                       │   │
│  │  ✗ LocalStorage files                                 │   │
│  │  ✗ IndexedDB files                                    │   │
│  │  ✗ History database                                   │   │
│  │  ✗ Session restore                                    │   │
│  │  ✗ Profile directory (minimal, read-only)            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Request Flow

```
User enters URL
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  1. UI Process                                               │
│     • Validate URL (no file://, data:// to external)        │
│     • Send to Broker via IPC                                 │
└─────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  2. Broker Process                                           │
│     • Generate synthetic identity for this request           │
│     • Synthesize headers (UA, Accept-Language, etc.)         │
│     • Generate request ID                                    │
│     • Forward to Network Process                             │
└─────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  3. Network Process                                          │
│     • Create NEW Tor circuit for this request                │
│     • Apply traffic shaping (padding, jitter)                │
│     • Normalize TLS fingerprint                              │
│     • Resolve DNS over Tor                                   │
│     • Make HTTPS request via Tor                             │
│     • Receive response                                       │
│     • Strip tracking headers from response                   │
│     • Forward to Broker                                      │
└─────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  4. Broker Process                                           │
│     • Validate response                                      │
│     • Strip cookies from response                            │
│     • Forward content + synthetic identity to Content        │
└─────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  5. Content Process                                          │
│     • Parse HTML                                             │
│     • Apply JS shims with synthetic identity                 │
│     • Render (with fingerprint defenses active)              │
│     • Sub-resources: each gets NEW request cycle             │
│     • Display to UI                                          │
└─────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  6. Tab Close / Navigation                                   │
│     • Destroy all Content Process memory                     │
│     • Clear RAM cache                                        │
│     • Synthetic identity discarded                           │
│     • No state persists                                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Security Boundaries

### Content Process Sandbox (Linux)

```c
// Allowed syscalls (seccomp-bpf whitelist)
read, write, close, mmap, munmap, mprotect,
brk, rt_sigaction, rt_sigprocmask, rt_sigreturn,
ioctl(limited), access, pipe, pipe2, dup, dup2,
clone(no CLONE_NEWNET), wait4, exit, exit_group,
futex, set_tid_address, clock_gettime(MONOTONIC only),
epoll_create, epoll_ctl, epoll_wait,
recvmsg(AF_UNIX only), sendmsg(AF_UNIX only)

// Denied (partial list)
socket, connect, bind, listen, accept,  // No network
open, openat, creat,                    // No filesystem
execve, fork,                           // No process creation
ptrace,                                 // No debugging
mount, umount,                          // No filesystem changes
```

### Network Process Sandbox

```c
// Allowed
socket, connect, read, write, close,
sendto, recvfrom, setsockopt, getsockopt,
epoll_*, select, poll,
clock_gettime, nanosleep,
mmap, munmap, brk

// Denied
open, openat,     // No local filesystem
fork, execve,     // No process creation
ptrace,           // No debugging
```

---

## Cryptographic Operations

### Random Number Generation

- Source: `/dev/urandom` (Linux) or equivalent
- Used for:
  - Synthetic identity generation
  - Timing jitter
  - Padding generation
  - Circuit selection assistance

### Key Management

- No persistent keys (no profiles)
- Tor keys managed by tor daemon
- TLS session keys: ephemeral, discarded after request

---

## Error Handling Philosophy

1. **Fail closed**: If any security mechanism fails, abort the request
2. **No fallback**: Never fall back to insecure methods
3. **Silent failure**: Don't reveal failure details to websites
4. **User notification**: Alert user to security issues
