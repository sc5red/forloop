//! WebGL fingerprinting defense.
//!
//! WebGL exposes GPU information that can fingerprint users.
//! We return generic values from a defined anonymity set.

/// WebGL defense configuration.
#[derive(Debug, Clone)]
pub struct WebGLDefense {
    /// Seed for this identity
    seed: u64,
    /// Selected profile
    profile: WebGLProfile,
}

/// WebGL profile representing a common configuration.
#[derive(Debug, Clone)]
pub struct WebGLProfile {
    /// Renderer string
    pub renderer: &'static str,
    /// Vendor string
    pub vendor: &'static str,
    /// Unmasked renderer
    pub unmasked_renderer: &'static str,
    /// Unmasked vendor
    pub unmasked_vendor: &'static str,
    /// Max texture size
    pub max_texture_size: i32,
    /// Max viewport dimensions
    pub max_viewport_dims: (i32, i32),
    /// Max vertex attribs
    pub max_vertex_attribs: i32,
    /// Max vertex uniform vectors
    pub max_vertex_uniform_vectors: i32,
    /// Max fragment uniform vectors
    pub max_fragment_uniform_vectors: i32,
    /// Max varying vectors
    pub max_varying_vectors: i32,
    /// Supported extensions
    pub extensions: &'static [&'static str],
}

/// Pre-defined WebGL profiles matching common configurations.
const WEBGL_PROFILES: &[WebGLProfile] = &[
    WebGLProfile {
        renderer: "WebKit WebGL",
        vendor: "WebKit",
        unmasked_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0)",
        unmasked_vendor: "Google Inc. (Intel)",
        max_texture_size: 16384,
        max_viewport_dims: (16384, 16384),
        max_vertex_attribs: 16,
        max_vertex_uniform_vectors: 4096,
        max_fragment_uniform_vectors: 1024,
        max_varying_vectors: 30,
        extensions: &[
            "ANGLE_instanced_arrays",
            "EXT_blend_minmax",
            "EXT_color_buffer_half_float",
            "EXT_float_blend",
            "EXT_frag_depth",
            "EXT_shader_texture_lod",
            "EXT_texture_filter_anisotropic",
            "OES_element_index_uint",
            "OES_standard_derivatives",
            "OES_texture_float",
            "OES_texture_float_linear",
            "OES_texture_half_float",
            "OES_texture_half_float_linear",
            "OES_vertex_array_object",
            "WEBGL_color_buffer_float",
            "WEBGL_compressed_texture_s3tc",
            "WEBGL_debug_renderer_info",
            "WEBGL_debug_shaders",
            "WEBGL_depth_texture",
            "WEBGL_draw_buffers",
            "WEBGL_lose_context",
        ],
    },
    WebGLProfile {
        renderer: "WebKit WebGL",
        vendor: "WebKit",
        unmasked_renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 Direct3D11 vs_5_0 ps_5_0)",
        unmasked_vendor: "Google Inc. (NVIDIA)",
        max_texture_size: 16384,
        max_viewport_dims: (32767, 32767),
        max_vertex_attribs: 16,
        max_vertex_uniform_vectors: 4096,
        max_fragment_uniform_vectors: 1024,
        max_varying_vectors: 31,
        extensions: &[
            "ANGLE_instanced_arrays",
            "EXT_blend_minmax",
            "EXT_color_buffer_half_float",
            "EXT_float_blend",
            "EXT_frag_depth",
            "EXT_shader_texture_lod",
            "EXT_texture_filter_anisotropic",
            "OES_element_index_uint",
            "OES_standard_derivatives",
            "OES_texture_float",
            "OES_texture_float_linear",
            "OES_texture_half_float",
            "OES_texture_half_float_linear",
            "OES_vertex_array_object",
            "WEBGL_color_buffer_float",
            "WEBGL_compressed_texture_s3tc",
            "WEBGL_debug_renderer_info",
            "WEBGL_depth_texture",
            "WEBGL_draw_buffers",
            "WEBGL_lose_context",
        ],
    },
    // Mesa profile for Linux
    WebGLProfile {
        renderer: "WebKit WebGL",
        vendor: "WebKit",
        unmasked_renderer: "Mesa DRI Intel(R) UHD Graphics 620 (KBL GT2)",
        unmasked_vendor: "Intel Open Source Technology Center",
        max_texture_size: 16384,
        max_viewport_dims: (16384, 16384),
        max_vertex_attribs: 16,
        max_vertex_uniform_vectors: 4096,
        max_fragment_uniform_vectors: 1024,
        max_varying_vectors: 31,
        extensions: &[
            "ANGLE_instanced_arrays",
            "EXT_blend_minmax",
            "EXT_frag_depth",
            "EXT_shader_texture_lod",
            "EXT_texture_filter_anisotropic",
            "OES_element_index_uint",
            "OES_standard_derivatives",
            "OES_texture_float",
            "OES_texture_float_linear",
            "OES_texture_half_float",
            "OES_texture_half_float_linear",
            "OES_vertex_array_object",
            "WEBGL_depth_texture",
            "WEBGL_draw_buffers",
            "WEBGL_lose_context",
        ],
    },
];

impl WebGLDefense {
    /// Create a new WebGL defense.
    pub fn new(seed: u64) -> Self {
        // Select profile based on seed
        let profile_idx = (seed as usize) % WEBGL_PROFILES.len();
        Self {
            seed,
            profile: WEBGL_PROFILES[profile_idx].clone(),
        }
    }

    /// Get the spoofed renderer string.
    pub fn renderer(&self) -> &str {
        self.profile.renderer
    }

    /// Get the spoofed vendor string.
    pub fn vendor(&self) -> &str {
        self.profile.vendor
    }

    /// Get the unmasked renderer (if debug info is "allowed").
    pub fn unmasked_renderer(&self) -> &str {
        self.profile.unmasked_renderer
    }

    /// Get the unmasked vendor.
    pub fn unmasked_vendor(&self) -> &str {
        self.profile.unmasked_vendor
    }

    /// Get a WebGL parameter value.
    pub fn get_parameter(&self, pname: u32) -> WebGLValue {
        match pname {
            // GL_MAX_TEXTURE_SIZE
            0x0D33 => WebGLValue::Int(self.profile.max_texture_size),
            // GL_MAX_VIEWPORT_DIMS
            0x0D3A => WebGLValue::IntVec2(
                self.profile.max_viewport_dims.0,
                self.profile.max_viewport_dims.1,
            ),
            // GL_MAX_VERTEX_ATTRIBS
            0x8869 => WebGLValue::Int(self.profile.max_vertex_attribs),
            // GL_MAX_VERTEX_UNIFORM_VECTORS
            0x8DFB => WebGLValue::Int(self.profile.max_vertex_uniform_vectors),
            // GL_MAX_FRAGMENT_UNIFORM_VECTORS
            0x8DFD => WebGLValue::Int(self.profile.max_fragment_uniform_vectors),
            // GL_MAX_VARYING_VECTORS
            0x8DFC => WebGLValue::Int(self.profile.max_varying_vectors),
            // GL_RENDERER
            0x1F01 => WebGLValue::String(self.profile.renderer.to_string()),
            // GL_VENDOR
            0x1F00 => WebGLValue::String(self.profile.vendor.to_string()),
            // Default: return null
            _ => WebGLValue::Null,
        }
    }

    /// Get supported extensions.
    pub fn supported_extensions(&self) -> Vec<String> {
        // Return a subset of extensions to reduce fingerprint surface
        vec![
            "OES_texture_float".to_string(),
            "OES_texture_half_float".to_string(),
            "OES_standard_derivatives".to_string(),
            "OES_element_index_uint".to_string(),
            "WEBGL_depth_texture".to_string(),
            "WEBGL_lose_context".to_string(),
        ]
    }

    /// Generate deterministic noise for readPixels.
    pub fn apply_pixel_noise(&self, data: &mut [u8]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        for (i, byte) in data.iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            self.seed.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();

            // Apply very subtle noise
            let noise = ((hash & 0x03) as i16) - 1; // -1, 0, 1, or 2
            *byte = (*byte as i16 + noise).clamp(0, 255) as u8;
        }
    }
}

/// WebGL value types.
#[derive(Debug, Clone)]
pub enum WebGLValue {
    /// Null value
    Null,
    /// Integer value
    Int(i32),
    /// Two integers
    IntVec2(i32, i32),
    /// Float value
    Float(f32),
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_selection() {
        let defense1 = WebGLDefense::new(0);
        let defense2 = WebGLDefense::new(1);
        let defense3 = WebGLDefense::new(3); // Wraps to 0

        // Same seed mod profiles should give same profile
        assert_eq!(defense1.renderer(), defense3.renderer());
    }

    #[test]
    fn test_get_parameter() {
        let defense = WebGLDefense::new(0);

        if let WebGLValue::Int(size) = defense.get_parameter(0x0D33) {
            assert!(size >= 8192);
        } else {
            panic!("Expected Int");
        }
    }

    #[test]
    fn test_pixel_noise_deterministic() {
        let defense = WebGLDefense::new(42);

        let mut data1 = vec![128u8; 64];
        let mut data2 = vec![128u8; 64];

        defense.apply_pixel_noise(&mut data1);
        defense.apply_pixel_noise(&mut data2);

        assert_eq!(data1, data2);
    }
}
