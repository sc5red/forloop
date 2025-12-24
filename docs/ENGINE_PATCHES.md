# forloop Engine Patches

This document specifies the exact patches required to Firefox/Gecko to implement forloop's privacy guarantees.

## Base: Firefox ESR 128.x (via LibreWolf)

We start with LibreWolf as it already removes telemetry and applies initial hardening.

## Patch Organization

```
patches/
├── 0001-disable-all-telemetry.patch
├── 0002-disable-crash-reporter.patch
├── 0003-block-all-storage.patch
├── 0004-disable-cache.patch
├── 0005-disable-service-workers.patch
├── 0006-disable-webrtc.patch
├── 0007-inject-js-shims.patch
├── 0008-header-stripping.patch
├── 0009-fingerprint-defense-integration.patch
├── 0010-network-isolation.patch
├── 0011-disable-sensors.patch
├── 0012-timing-fuzzing.patch
└── 0013-disable-history.patch
```

---

## Patch 0001: Disable All Telemetry

**File: `toolkit/components/telemetry/TelemetryController.jsm`**

```diff
--- a/toolkit/components/telemetry/TelemetryController.jsm
+++ b/toolkit/components/telemetry/TelemetryController.jsm
@@ -1,6 +1,10 @@
 /* This Source Code Form is subject to the terms of the Mozilla Public
  * License, v. 2.0. */

+// FORLOOP: Telemetry completely disabled
+// No data collection of any kind
+
 const EXPORTED_SYMBOLS = ["TelemetryController"];

 const TelemetryController = {
+  // All methods are no-ops
   observe() {},
   submitExternalPing() { return Promise.resolve(); },
   getCurrentPingData() { return {}; },
@@ -10,6 +14,7 @@
   setServer() {},
   testReset() {},
   testSetupContent() { return Promise.resolve(); },
+  // FORLOOP: Prevent any initialization
   setupContent() { return Promise.resolve(); },
   setup() { return Promise.resolve(); },
   shutdown() { return Promise.resolve(); },
 };
```

---

## Patch 0002: Disable Crash Reporter

**File: `toolkit/crashreporter/nsExceptionHandler.cpp`**

```diff
--- a/toolkit/crashreporter/nsExceptionHandler.cpp
+++ b/toolkit/crashreporter/nsExceptionHandler.cpp
@@ -100,6 +100,11 @@
 namespace CrashReporter {

 bool GetEnabled() {
+  // FORLOOP: Crash reporter permanently disabled
+  // No crash data is ever collected or sent
+  return false;
+
+#if 0  // Original code disabled
   return gExceptionHandler != nullptr;
+#endif
 }

 bool SetEnabled(bool aEnabled) {
+  // FORLOOP: Cannot be enabled
+  return false;
 }
```

---

## Patch 0003: Block All Storage APIs

**File: `dom/storage/StorageManager.cpp`**

```diff
--- a/dom/storage/StorageManager.cpp
+++ b/dom/storage/StorageManager.cpp
@@ -50,6 +50,12 @@
 already_AddRefed<Storage>
 StorageManager::GetStorage(nsPIDOMWindowInner* aWindow,
                           nsIPrincipal* aPrincipal) {
+  // FORLOOP: All storage APIs blocked
+  // localStorage, sessionStorage return null
+  // This prevents any persistent or session storage
+  aError.ThrowSecurityError("Storage is disabled");
+  return nullptr;
+
   // Original implementation removed
 }
```

**File: `dom/indexedDB/IDBFactory.cpp`**

```diff
--- a/dom/indexedDB/IDBFactory.cpp
+++ b/dom/indexedDB/IDBFactory.cpp
@@ -200,6 +200,11 @@
 already_AddRefed<IDBOpenDBRequest>
 IDBFactory::Open(const nsAString& aName, uint64_t aVersion,
                  ErrorResult& aRv) {
+  // FORLOOP: IndexedDB completely disabled
+  // No persistent storage allowed
+  aRv.ThrowSecurityError("IndexedDB is disabled");
+  return nullptr;
+
   // Original implementation removed
 }
```

---

## Patch 0004: Disable HTTP Cache

**File: `netwerk/cache2/CacheStorageService.cpp`**

```diff
--- a/netwerk/cache2/CacheStorageService.cpp
+++ b/netwerk/cache2/CacheStorageService.cpp
@@ -150,6 +150,15 @@
 nsresult
 CacheStorageService::Dispatch(CacheIOThread* aIOThread) {
+  // FORLOOP: Cache is RAM-only and per-request
+  // Nothing is written to disk, ever
+  // Cache entries are discarded after each request
+  return NS_ERROR_NOT_AVAILABLE;
+}
+
+nsresult
+CacheStorageService::AddStorageEntry(CacheStorage* aStorage,
+                                     const nsACString& aURI,
+                                     const nsACString& aIdExtension,
+                                     bool aCreate,
+                                     CacheEntryHandle** aResult) {
+  // FORLOOP: No cache entries created
+  return NS_ERROR_NOT_AVAILABLE;
 }
```

---

## Patch 0005: Disable Service Workers

**File: `dom/serviceworkers/ServiceWorkerManager.cpp`**

```diff
--- a/dom/serviceworkers/ServiceWorkerManager.cpp
+++ b/dom/serviceworkers/ServiceWorkerManager.cpp
@@ -300,6 +300,13 @@
 nsresult
 ServiceWorkerManager::Register(nsPIDOMWindowInner* aWindow,
                                const nsAString& aScriptURL,
                                const RegistrationOptions& aOptions) {
+  // FORLOOP: Service Workers completely disabled
+  // They enable offline storage and can persist state
+  return NS_ERROR_DOM_SECURITY_ERR;
+
   // Original implementation removed
 }
+
+bool
+ServiceWorkerManager::IsAvailable() {
+  // FORLOOP: Report as unavailable
+  return false;
+}
```

---

## Patch 0006: Disable WebRTC

**File: `dom/media/webrtc/PeerConnectionImpl.cpp`**

```diff
--- a/dom/media/webrtc/PeerConnectionImpl.cpp
+++ b/dom/media/webrtc/PeerConnectionImpl.cpp
@@ -100,6 +100,14 @@
 PeerConnectionImpl::PeerConnectionImpl() {
+  // FORLOOP: WebRTC is completely disabled
+  // This prevents IP leaks via STUN/TURN
+  MOZ_CRASH("WebRTC is disabled in forloop");
+}
+
+nsresult
+PeerConnectionImpl::Initialize(/* ... */) {
+  // FORLOOP: Prevent initialization
+  return NS_ERROR_NOT_AVAILABLE;
 }
```

**File: `dom/webidl/RTCPeerConnection.webidl`**

```diff
--- a/dom/webidl/RTCPeerConnection.webidl
+++ b/dom/webidl/RTCPeerConnection.webidl
@@ -1,5 +1,8 @@
 /* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */

+// FORLOOP: This interface is disabled
+// Attempting to construct throws

 [Exposed=Window,
- Pref="media.peerconnection.enabled"]
+ Pref="media.peerconnection.enabled",
+ Func="forloop::IsWebRTCEnabled"]
 interface RTCPeerConnection : EventTarget {
```

---

## Patch 0007: Inject Privacy Shims

**File: `dom/base/nsGlobalWindowInner.cpp`**

```diff
--- a/dom/base/nsGlobalWindowInner.cpp
+++ b/dom/base/nsGlobalWindowInner.cpp
@@ -1500,6 +1500,35 @@
 void
 nsGlobalWindowInner::InitDocumentDependentState(JSContext* aCx) {
+  // FORLOOP: Inject privacy shims before any page scripts run
+  InjectForloopPrivacyShims(aCx);
+
   // Original implementation continues...
 }
+
+void
+nsGlobalWindowInner::InjectForloopPrivacyShims(JSContext* aCx) {
+  // Generate synthetic identity for this request
+  ForloopIdentity identity = GenerateForloopIdentity();
+
+  // Inject identity as temporary global
+  JS::RootedObject global(aCx, GetGlobalJSObject());
+  JS::RootedValue identityVal(aCx);
+  if (!IdentityToJSValue(aCx, identity, &identityVal)) {
+    return;
+  }
+  JS_SetProperty(aCx, global, "__FORLOOP_IDENTITY__", identityVal);
+
+  // Execute privacy shims script
+  // This script is compiled into the binary
+  const char* shimsScript = GetForloopPrivacyShimsScript();
+  JS::CompileOptions options(aCx);
+  options.setFileAndLine("forloop://privacy-shims.js", 1);
+  
+  JS::RootedScript script(aCx);
+  if (JS::Compile(aCx, options, shimsScript, strlen(shimsScript), &script)) {
+    JS::RootedValue rval(aCx);
+    JS_ExecuteScript(aCx, script, &rval);
+  }
+}
```

---

## Patch 0008: Header Stripping

**File: `netwerk/protocol/http/nsHttpChannel.cpp`**

```diff
--- a/netwerk/protocol/http/nsHttpChannel.cpp
+++ b/netwerk/protocol/http/nsHttpChannel.cpp
@@ -1000,6 +1000,45 @@
 nsresult
 nsHttpChannel::SetupTransaction() {
+  // FORLOOP: Strip all identifying headers
+  StripIdentifyingHeaders();
+
   // Original implementation continues...
 }
+
+void
+nsHttpChannel::StripIdentifyingHeaders() {
+  // Remove headers that can identify the user
+  static const char* const headersToRemove[] = {
+    "Cookie",
+    "Referer",
+    "Origin",
+    "X-Forwarded-For",
+    "X-Real-IP",
+    "X-Client-IP",
+    "Via",
+    "Forwarded",
+    "DNT",
+    "Authorization",
+    "Proxy-Authorization",
+    nullptr
+  };
+
+  for (const char* const* header = headersToRemove; *header; ++header) {
+    mRequestHead.ClearHeader(nsHttp::ResolveAtom(*header));
+  }
+
+  // Set synthetic headers
+  ForloopIdentity identity = GetCurrentForloopIdentity();
+  mRequestHead.SetHeader(nsHttp::User_Agent, identity.userAgent);
+  mRequestHead.SetHeader(nsHttp::Accept_Language, "en-US,en;q=0.5"_ns);
+}
```

---

## Patch 0009: Fingerprint Defense Integration

**File: `dom/canvas/CanvasRenderingContext2D.cpp`**

```diff
--- a/dom/canvas/CanvasRenderingContext2D.cpp
+++ b/dom/canvas/CanvasRenderingContext2D.cpp
@@ -2000,6 +2000,25 @@
 already_AddRefed<ImageData>
 CanvasRenderingContext2D::GetImageData(int32_t aSx, int32_t aSy,
                                        int32_t aSw, int32_t aSh,
                                        ErrorResult& aError) {
+  // FORLOOP: Add deterministic noise to canvas data
+  RefPtr<ImageData> imageData = GetImageDataInternal(aSx, aSy, aSw, aSh, aError);
+  if (aError.Failed() || !imageData) {
+    return imageData.forget();
+  }
+
+  // Apply fingerprint defense noise
+  uint64_t seed = GetForloopCanvasSeed();
+  uint8_t* data = imageData->GetData();
+  uint32_t length = imageData->Length();
+
+  for (uint32_t i = 0; i < length; i += 4) {
+    uint64_t hash = HashPosition(seed, i / 4);
+    for (int c = 0; c < 3; c++) {  // RGB only, not alpha
+      int8_t noise = (hash >> (c * 8)) % 3 - 1;  // -1, 0, or 1
+      data[i + c] = ClampByte(data[i + c] + noise);
+    }
+  }
+
+  return imageData.forget();
 }
```

---

## Patch 0010: Network Isolation

**File: `netwerk/base/nsSocketTransportService2.cpp`**

```diff
--- a/netwerk/base/nsSocketTransportService2.cpp
+++ b/netwerk/base/nsSocketTransportService2.cpp
@@ -500,6 +500,20 @@
 nsresult
 nsSocketTransportService::CreateTransport(/* ... */) {
+  // FORLOOP: All connections must go through anonymization layer
+  // Direct connections are not allowed
+
+  // Check if this is going through our SOCKS5 proxy
+  if (!IsForloopProxyConfigured()) {
+    // Force all connections through Tor SOCKS5
+    proxyInfo = CreateForloopProxyInfo();
+  }
+
   // Original implementation with proxy enforcement
 }
+
+already_AddRefed<nsIProxyInfo>
+CreateForloopProxyInfo() {
+  // Create SOCKS5 proxy info pointing to embedded Tor
+  // Port 9150 on localhost
+  return CreateSOCKS5ProxyInfo("127.0.0.1"_ns, 9150);
+}
```

---

## Patch 0011: Disable Sensors

**File: `dom/base/Navigator.cpp`**

```diff
--- a/dom/base/Navigator.cpp
+++ b/dom/base/Navigator.cpp
@@ -800,6 +800,15 @@
 already_AddRefed<Promise>
 Navigator::GetBattery(ErrorResult& aRv) {
+  // FORLOOP: Battery API disabled
+  aRv.ThrowSecurityError("Battery API is disabled");
+  return nullptr;
 }

+Geolocation*
+Navigator::GetGeolocation(ErrorResult& aRv) {
+  // FORLOOP: Geolocation disabled
+  aRv.ThrowSecurityError("Geolocation is disabled");
+  return nullptr;
+}
```

---

## Patch 0012: Timing Fuzzing

**File: `js/src/vm/DateObject.cpp`**

```diff
--- a/js/src/vm/DateObject.cpp
+++ b/js/src/vm/DateObject.cpp
@@ -100,6 +100,18 @@
 static double
 NowAsMillis(JSContext* cx) {
+  // FORLOOP: Reduce timing precision to 100ms
+  // This prevents timing-based fingerprinting and side-channel attacks
+  double now = OriginalNowAsMillis();
+  
+  const double precision = 100.0;  // 100ms buckets
+  double reduced = floor(now / precision) * precision;
+  
+  // Add deterministic jitter based on request identity
+  uint64_t jitter = GetForloopTimingJitter() % 10;
+  
+  return reduced + jitter;
 }
```

**File: `dom/performance/Performance.cpp`**

```diff
--- a/dom/performance/Performance.cpp
+++ b/dom/performance/Performance.cpp
@@ -200,6 +200,15 @@
 DOMHighResTimeStamp
 Performance::Now() const {
+  // FORLOOP: Reduce precision to 100ms
+  // Matches Tor Browser's approach
+  DOMHighResTimeStamp now = OriginalNow();
+  
+  const double precision = 100.0;
+  DOMHighResTimeStamp reduced = floor(now / precision) * precision;
+  
+  return reduced + GetForloopTimingJitter() % 10;
 }
```

---

## Patch 0013: Disable History

**File: `toolkit/components/places/nsNavHistory.cpp`**

```diff
--- a/toolkit/components/places/nsNavHistory.cpp
+++ b/toolkit/components/places/nsNavHistory.cpp
@@ -500,6 +500,12 @@
 nsresult
 nsNavHistory::AddVisit(nsIURI* aURI, /* ... */) {
+  // FORLOOP: No history recorded
+  // This method is a no-op
+  return NS_OK;
+}
+
+nsresult
+nsNavHistory::AddPageToSession(/* ... */) {
+  // FORLOOP: No session history
+  return NS_OK;
 }
```

---

## about:config Overrides

In addition to C++ patches, these preferences are locked:

```javascript
// FILE: forloop/defaults/pref/forloop.js

// Disable all telemetry
lockPref("toolkit.telemetry.enabled", false);
lockPref("toolkit.telemetry.unified", false);
lockPref("toolkit.telemetry.archive.enabled", false);

// Disable crash reporter
lockPref("breakpad.reportURL", "");
lockPref("browser.crashReports.unsubmittedCheck.enabled", false);

// Disable safe browsing (phones home)
lockPref("browser.safebrowsing.enabled", false);
lockPref("browser.safebrowsing.malware.enabled", false);
lockPref("browser.safebrowsing.phishing.enabled", false);

// Disable geolocation
lockPref("geo.enabled", false);

// Disable WebRTC
lockPref("media.peerconnection.enabled", false);
lockPref("media.navigator.enabled", false);

// Disable DRM
lockPref("media.eme.enabled", false);

// Disable pocket
lockPref("extensions.pocket.enabled", false);

// Disable suggestions
lockPref("browser.search.suggest.enabled", false);
lockPref("browser.urlbar.suggest.searches", false);

// Disable prefetching
lockPref("network.prefetch-next", false);
lockPref("network.dns.disablePrefetch", true);
lockPref("network.predictor.enabled", false);

// Force HTTPS
lockPref("dom.security.https_only_mode", true);
lockPref("dom.security.https_only_mode_ever_enabled", true);

// Disable Service Workers
lockPref("dom.serviceWorkers.enabled", false);

// Disable IndexedDB
lockPref("dom.indexedDB.enabled", false);

// Disable localStorage
lockPref("dom.storage.enabled", false);

// Reduce timing precision
lockPref("privacy.reduceTimerPrecision", true);
lockPref("privacy.resistFingerprinting.reduceTimerPrecision.microseconds", 100000);

// Fingerprinting protection
lockPref("privacy.resistFingerprinting", true);
lockPref("privacy.resistFingerprinting.letterboxing", true);

// Disable cache
lockPref("browser.cache.disk.enable", false);
lockPref("browser.cache.memory.enable", true);
lockPref("browser.cache.memory.capacity", 0);

// Clear everything on shutdown
lockPref("privacy.sanitize.sanitizeOnShutdown", true);
lockPref("privacy.clearOnShutdown.cache", true);
lockPref("privacy.clearOnShutdown.cookies", true);
lockPref("privacy.clearOnShutdown.downloads", true);
lockPref("privacy.clearOnShutdown.formdata", true);
lockPref("privacy.clearOnShutdown.history", true);
lockPref("privacy.clearOnShutdown.sessions", true);

// Disable all cookies
lockPref("network.cookie.cookieBehavior", 2);  // Block all

// Isolate requests
lockPref("privacy.firstparty.isolate", true);

// Disable speculative connections
lockPref("network.http.speculative-parallel-limit", 0);

// Use Tor SOCKS proxy
lockPref("network.proxy.type", 1);
lockPref("network.proxy.socks", "127.0.0.1");
lockPref("network.proxy.socks_port", 9150);
lockPref("network.proxy.socks_remote_dns", true);
lockPref("network.proxy.no_proxies_on", "");  // Everything through proxy

// Disable auto-updates (handled by our update system)
lockPref("app.update.enabled", false);
lockPref("app.update.auto", false);

// Disable extension updates
lockPref("extensions.update.enabled", false);

// Disable captive portal detection
lockPref("network.captive-portal-service.enabled", false);

// Disable network connectivity checks
lockPref("network.connectivity-service.enabled", false);
```

---

## Verification

After applying patches, verify:

1. `navigator.userAgent` returns synthetic UA
2. `Date.now()` has 100ms precision
3. `localStorage` throws SecurityError
4. `RTCPeerConnection` is undefined
5. Canvas toDataURL returns consistent fake data
6. All requests go through SOCKS5 proxy
7. No DNS queries bypass proxy
8. No telemetry endpoints are contacted
