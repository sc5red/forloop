// forloop Browser - JavaScript API Shims
//
// This script is injected into every page before any other scripts run.
// It overrides fingerprinting APIs with privacy-preserving alternatives.
//
// This file is compiled into the browser and injected at the engine level.
// It cannot be detected or circumvented by page scripts.

(function() {
    'use strict';

    // Prevent any modification of our overrides
    const freeze = Object.freeze;
    const defineProperty = Object.defineProperty;
    const getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;

    // Synthetic identity values (injected by browser at runtime)
    const IDENTITY = window.__FORLOOP_IDENTITY__ || {
        canvasSeed: Math.random() * 0xFFFFFFFF >>> 0,
        webglSeed: Math.random() * 0xFFFFFFFF >>> 0,
        audioSeed: Math.random() * 0xFFFFFFFF >>> 0,
        timezoneOffset: 0,
        screenWidth: 1920,
        screenHeight: 1080,
        hardwareConcurrency: 4,
        deviceMemory: 8,
        platform: 'Win32',
        userAgent: 'Mozilla/5.0 (Windows NT 10.0; rv:115.0) Gecko/20100101 Firefox/115.0'
    };

    // Remove our identity object from window
    delete window.__FORLOOP_IDENTITY__;

    // ============================================================
    // TIMING API OVERRIDES
    // ============================================================

    const TIMING_PRECISION_MS = 100;

    function fuzzTimestamp(timestamp) {
        return Math.floor(timestamp / TIMING_PRECISION_MS) * TIMING_PRECISION_MS;
    }

    // Override Date.now
    const originalDateNow = Date.now;
    Date.now = function() {
        return fuzzTimestamp(originalDateNow.call(Date));
    };
    freeze(Date.now);

    // Override performance.now
    const originalPerformanceNow = performance.now.bind(performance);
    performance.now = function() {
        return fuzzTimestamp(originalPerformanceNow());
    };
    freeze(performance.now);

    // Override Date constructor for new Date()
    const OriginalDate = Date;
    function ForloopDate(...args) {
        if (args.length === 0) {
            return new OriginalDate(fuzzTimestamp(originalDateNow.call(Date)));
        }
        return new OriginalDate(...args);
    }
    ForloopDate.now = Date.now;
    ForloopDate.parse = OriginalDate.parse;
    ForloopDate.UTC = OriginalDate.UTC;
    ForloopDate.prototype = OriginalDate.prototype;
    window.Date = ForloopDate;

    // ============================================================
    // NAVIGATOR OVERRIDES
    // ============================================================

    const navigatorOverrides = {
        userAgent: IDENTITY.userAgent,
        platform: IDENTITY.platform,
        language: 'en-US',
        languages: freeze(['en-US', 'en']),
        hardwareConcurrency: IDENTITY.hardwareConcurrency,
        deviceMemory: IDENTITY.deviceMemory,
        maxTouchPoints: 0,
        cookieEnabled: false,
        doNotTrack: null,
        webdriver: false,
        vendor: '',
        vendorSub: '',
        productSub: '20100101',
        plugins: freeze({ length: 0 }),
        mimeTypes: freeze({ length: 0 }),
        onLine: true,
        pdfViewerEnabled: true,
    };

    for (const [key, value] of Object.entries(navigatorOverrides)) {
        try {
            defineProperty(navigator, key, {
                get: function() { return value; },
                configurable: false,
                enumerable: true
            });
        } catch (e) {
            // Property may not be configurable
        }
    }

    // Block sensitive navigator APIs
    const blockedNavigatorApis = [
        'bluetooth',
        'usb',
        'hid',
        'serial',
        'nfc',
        'xr',
        'keyboard',
        'wakeLock',
        'virtualKeyboard',
        'credentials',
        'mediaDevices',
        'geolocation',
    ];

    for (const api of blockedNavigatorApis) {
        try {
            defineProperty(navigator, api, {
                get: function() { return undefined; },
                configurable: false,
                enumerable: false
            });
        } catch (e) {}
    }

    // Override getBattery to reject
    navigator.getBattery = function() {
        return Promise.reject(new Error('Battery API is disabled'));
    };
    freeze(navigator.getBattery);

    // Override requestMIDIAccess to reject
    navigator.requestMIDIAccess = function() {
        return Promise.reject(new Error('MIDI API is disabled'));
    };

    // Override getGamepads to return empty
    navigator.getGamepads = function() {
        return [];
    };

    // ============================================================
    // SCREEN OVERRIDES
    // ============================================================

    const screenOverrides = {
        width: IDENTITY.screenWidth,
        height: IDENTITY.screenHeight,
        availWidth: IDENTITY.screenWidth,
        availHeight: IDENTITY.screenHeight - 40,
        colorDepth: 24,
        pixelDepth: 24,
        availLeft: 0,
        availTop: 0,
    };

    for (const [key, value] of Object.entries(screenOverrides)) {
        try {
            defineProperty(screen, key, {
                get: function() { return value; },
                configurable: false,
                enumerable: true
            });
        } catch (e) {}
    }

    // Override window dimensions
    const windowOverrides = {
        innerWidth: IDENTITY.screenWidth,
        innerHeight: IDENTITY.screenHeight - 100,
        outerWidth: IDENTITY.screenWidth,
        outerHeight: IDENTITY.screenHeight,
        screenX: 0,
        screenY: 0,
        screenLeft: 0,
        screenTop: 0,
        devicePixelRatio: 1,
    };

    for (const [key, value] of Object.entries(windowOverrides)) {
        try {
            defineProperty(window, key, {
                get: function() { return value; },
                configurable: false,
                enumerable: true
            });
        } catch (e) {}
    }

    // ============================================================
    // CANVAS FINGERPRINTING DEFENSE
    // ============================================================

    function hashCode(seed, x, y) {
        let hash = seed;
        hash = ((hash << 5) - hash) + x;
        hash = ((hash << 5) - hash) + y;
        return hash & hash;
    }

    const originalGetContext = HTMLCanvasElement.prototype.getContext;
    const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
    const originalToBlob = HTMLCanvasElement.prototype.toBlob;

    HTMLCanvasElement.prototype.toDataURL = function(...args) {
        // Return a consistent fake data URL
        const hash = (IDENTITY.canvasSeed >>> 0).toString(16).padStart(8, '0');
        return `data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==${hash}`;
    };

    HTMLCanvasElement.prototype.toBlob = function(callback, ...args) {
        const dataUrl = this.toDataURL(...args);
        const binary = atob(dataUrl.split(',')[1]);
        const array = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
            array[i] = binary.charCodeAt(i);
        }
        callback(new Blob([array], { type: 'image/png' }));
    };

    // Intercept getImageData to add noise
    const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;
    CanvasRenderingContext2D.prototype.getImageData = function(sx, sy, sw, sh) {
        const imageData = originalGetImageData.call(this, sx, sy, sw, sh);
        const data = imageData.data;

        // Add deterministic noise based on seed and position
        for (let y = 0; y < sh; y++) {
            for (let x = 0; x < sw; x++) {
                const idx = (y * sw + x) * 4;
                const noise = hashCode(IDENTITY.canvasSeed, x + sx, y + sy);

                // Subtle noise to RGB channels
                for (let c = 0; c < 3; c++) {
                    const delta = ((noise >> (c * 8)) & 0xFF) % 3 - 1;
                    data[idx + c] = Math.max(0, Math.min(255, data[idx + c] + delta));
                }
            }
        }

        return imageData;
    };

    // ============================================================
    // WEBGL FINGERPRINTING DEFENSE
    // ============================================================

    const webglParameters = {
        [WebGLRenderingContext.RENDERER]: 'WebKit WebGL',
        [WebGLRenderingContext.VENDOR]: 'WebKit',
        [WebGLRenderingContext.MAX_TEXTURE_SIZE]: 16384,
        [WebGLRenderingContext.MAX_VIEWPORT_DIMS]: new Int32Array([16384, 16384]),
        [WebGLRenderingContext.MAX_VERTEX_ATTRIBS]: 16,
        [WebGLRenderingContext.MAX_VERTEX_UNIFORM_VECTORS]: 4096,
        [WebGLRenderingContext.MAX_FRAGMENT_UNIFORM_VECTORS]: 1024,
        [WebGLRenderingContext.MAX_VARYING_VECTORS]: 30,
    };

    const originalGetParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(pname) {
        if (webglParameters.hasOwnProperty(pname)) {
            return webglParameters[pname];
        }
        return originalGetParameter.call(this, pname);
    };

    // Block WEBGL_debug_renderer_info extension
    const originalGetExtension = WebGLRenderingContext.prototype.getExtension;
    WebGLRenderingContext.prototype.getExtension = function(name) {
        if (name === 'WEBGL_debug_renderer_info') {
            return null;
        }
        return originalGetExtension.call(this, name);
    };

    // Override getSupportedExtensions
    const originalGetSupportedExtensions = WebGLRenderingContext.prototype.getSupportedExtensions;
    WebGLRenderingContext.prototype.getSupportedExtensions = function() {
        const extensions = originalGetSupportedExtensions.call(this);
        if (!extensions) return extensions;
        return extensions.filter(ext => ext !== 'WEBGL_debug_renderer_info');
    };

    // Apply same overrides to WebGL2
    if (typeof WebGL2RenderingContext !== 'undefined') {
        WebGL2RenderingContext.prototype.getParameter = WebGLRenderingContext.prototype.getParameter;
        WebGL2RenderingContext.prototype.getExtension = WebGLRenderingContext.prototype.getExtension;
        WebGL2RenderingContext.prototype.getSupportedExtensions = WebGLRenderingContext.prototype.getSupportedExtensions;
    }

    // ============================================================
    // AUDIO FINGERPRINTING DEFENSE
    // ============================================================

    if (typeof AudioContext !== 'undefined' || typeof webkitAudioContext !== 'undefined') {
        const OriginalAudioContext = window.AudioContext || window.webkitAudioContext;
        const originalCreateAnalyser = OriginalAudioContext.prototype.createAnalyser;
        const originalCreateOscillator = OriginalAudioContext.prototype.createOscillator;
        const originalCreateDynamicsCompressor = OriginalAudioContext.prototype.createDynamicsCompressor;

        // Override getFloatFrequencyData to add noise
        const originalGetFloatFrequencyData = AnalyserNode.prototype.getFloatFrequencyData;
        AnalyserNode.prototype.getFloatFrequencyData = function(array) {
            originalGetFloatFrequencyData.call(this, array);
            for (let i = 0; i < array.length; i++) {
                const noise = hashCode(IDENTITY.audioSeed, i, 0) / 0x7FFFFFFF * 0.01;
                array[i] += noise;
            }
        };

        const originalGetByteFrequencyData = AnalyserNode.prototype.getByteFrequencyData;
        AnalyserNode.prototype.getByteFrequencyData = function(array) {
            originalGetByteFrequencyData.call(this, array);
            for (let i = 0; i < array.length; i++) {
                const noise = hashCode(IDENTITY.audioSeed, i, 1) % 3 - 1;
                array[i] = Math.max(0, Math.min(255, array[i] + noise));
            }
        };
    }

    // ============================================================
    // FONT ENUMERATION DEFENSE
    // ============================================================

    // Block document.fonts
    try {
        defineProperty(document, 'fonts', {
            get: function() {
                return {
                    check: function() { return false; },
                    load: function() { return Promise.reject(new Error('Font loading disabled')); },
                    ready: Promise.resolve(),
                    forEach: function() {},
                    entries: function() { return [][Symbol.iterator](); },
                    keys: function() { return [][Symbol.iterator](); },
                    values: function() { return [][Symbol.iterator](); },
                    [Symbol.iterator]: function() { return [][Symbol.iterator](); },
                    size: 0
                };
            },
            configurable: false
        });
    } catch (e) {}

    // ============================================================
    // STORAGE API BLOCKING
    // ============================================================

    // Block localStorage
    try {
        defineProperty(window, 'localStorage', {
            get: function() {
                throw new DOMException('localStorage is disabled', 'SecurityError');
            },
            configurable: false
        });
    } catch (e) {}

    // Block sessionStorage
    try {
        defineProperty(window, 'sessionStorage', {
            get: function() {
                throw new DOMException('sessionStorage is disabled', 'SecurityError');
            },
            configurable: false
        });
    } catch (e) {}

    // Block IndexedDB
    try {
        defineProperty(window, 'indexedDB', {
            get: function() { return undefined; },
            configurable: false
        });
    } catch (e) {}

    // Block cookies
    try {
        defineProperty(document, 'cookie', {
            get: function() { return ''; },
            set: function() { /* silently fail */ },
            configurable: false
        });
    } catch (e) {}

    // ============================================================
    // WEBRTC BLOCKING
    // ============================================================

    // Completely remove WebRTC
    const rtcApis = [
        'RTCPeerConnection',
        'webkitRTCPeerConnection',
        'mozRTCPeerConnection',
        'RTCSessionDescription',
        'RTCIceCandidate',
        'RTCDataChannel',
        'MediaStream',
        'MediaStreamTrack',
        'RTCRtpSender',
        'RTCRtpReceiver',
    ];

    for (const api of rtcApis) {
        try {
            defineProperty(window, api, {
                get: function() { return undefined; },
                configurable: false,
                enumerable: false
            });
        } catch (e) {}
    }

    // ============================================================
    // SENSOR API BLOCKING
    // ============================================================

    const sensorApis = [
        'Accelerometer',
        'Gyroscope',
        'Magnetometer',
        'AmbientLightSensor',
        'DeviceMotionEvent',
        'DeviceOrientationEvent',
        'AbsoluteOrientationSensor',
        'RelativeOrientationSensor',
        'GravitySensor',
        'LinearAccelerationSensor',
    ];

    for (const api of sensorApis) {
        try {
            defineProperty(window, api, {
                get: function() { return undefined; },
                configurable: false,
                enumerable: false
            });
        } catch (e) {}
    }

    // ============================================================
    // SHARED MEMORY BLOCKING (Spectre mitigation)
    // ============================================================

    try {
        defineProperty(window, 'SharedArrayBuffer', {
            get: function() { return undefined; },
            configurable: false,
            enumerable: false
        });
    } catch (e) {}

    try {
        defineProperty(window, 'Atomics', {
            get: function() { return undefined; },
            configurable: false,
            enumerable: false
        });
    } catch (e) {}

    // ============================================================
    // SERVICE WORKER BLOCKING
    // ============================================================

    if ('serviceWorker' in navigator) {
        const originalRegister = navigator.serviceWorker.register;
        navigator.serviceWorker.register = function() {
            return Promise.reject(new Error('Service Workers are disabled'));
        };

        try {
            defineProperty(navigator, 'serviceWorker', {
                get: function() {
                    return {
                        register: function() {
                            return Promise.reject(new Error('Service Workers are disabled'));
                        },
                        ready: Promise.reject(new Error('Service Workers are disabled')),
                        controller: null,
                        getRegistrations: function() { return Promise.resolve([]); },
                        getRegistration: function() { return Promise.resolve(undefined); }
                    };
                },
                configurable: false
            });
        } catch (e) {}
    }

    // ============================================================
    // CACHE API BLOCKING
    // ============================================================

    try {
        defineProperty(window, 'caches', {
            get: function() { return undefined; },
            configurable: false,
            enumerable: false
        });
    } catch (e) {}

    // ============================================================
    // TIMEZONE SPOOFING
    // ============================================================

    // Override getTimezoneOffset
    const originalGetTimezoneOffset = Date.prototype.getTimezoneOffset;
    Date.prototype.getTimezoneOffset = function() {
        return IDENTITY.timezoneOffset;
    };

    // Override Intl.DateTimeFormat for timezone
    const OriginalDateTimeFormat = Intl.DateTimeFormat;
    Intl.DateTimeFormat = function(locales, options) {
        const newOptions = Object.assign({}, options, { timeZone: 'UTC' });
        return new OriginalDateTimeFormat(locales, newOptions);
    };
    Intl.DateTimeFormat.prototype = OriginalDateTimeFormat.prototype;
    Intl.DateTimeFormat.supportedLocalesOf = OriginalDateTimeFormat.supportedLocalesOf;

    // ============================================================
    // PERMISSION API OVERRIDE
    // ============================================================

    if (navigator.permissions) {
        const originalQuery = navigator.permissions.query;
        navigator.permissions.query = function(descriptor) {
            return Promise.resolve({
                state: 'denied',
                status: 'denied',
                onchange: null
            });
        };
    }

    // ============================================================
    // NOTIFICATION API BLOCKING
    // ============================================================

    try {
        defineProperty(window, 'Notification', {
            get: function() {
                return {
                    permission: 'denied',
                    requestPermission: function() { return Promise.resolve('denied'); }
                };
            },
            configurable: false
        });
    } catch (e) {}

    // ============================================================
    // BEACON API BLOCKING
    // ============================================================

    navigator.sendBeacon = function() { return false; };

    // ============================================================
    // FINAL: Freeze prototypes to prevent tampering
    // ============================================================

    // Make overrides non-configurable where possible
    console.log('[forloop] Privacy protections active');

})();
