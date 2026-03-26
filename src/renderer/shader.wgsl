// src/renderer/shader.wgsl

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct FilterUniform {
    /// 3x3 Color conversion matrix for gamut mapping.
    /// Matched with Rust FilterUniform [[f32; 4]; 3] layout.
    color_matrix: mat3x3<f32>,

    bright: f32,
    contrast: f32,
    gamma: f32,
    exposure: f32,

    fsr_enabled: f32,
    icc_gamma: f32,
    fsr_sharpness: f32,
    median_enabled: f32,

    median_strength: f32,
    median_stride: f32,
    blur_radius: f32,
    unsharp_amount: f32,

    unsharp_threshold: f32,
    levels_in_black: f32,
    levels_in_white: f32,
    levels_gamma: f32,

    levels_out_black: f32,
    levels_out_white: f32,
    bypass_color: f32,
    bypass_median: f32,

    bypass_fsr: f32,
    bypass_detail: f32,
    bypass_levels: f32,
    _pad0: f32,
};

@group(2) @binding(0)
var<uniform> filters: FilterUniform;

fn get_brightness(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_sampled = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let dims = vec2<f32>(textureDimensions(t_diffuse));
    let texel = vec2<f32>(1.0) / max(dims, vec2<f32>(1.0));
    
    var rgb: vec3<f32>;

    // 1. AMD FSR 1.0 EASU (Edge Adaptive Spatial Upsampling) - Enhanced 12-tap
    if (filters.fsr_enabled > 0.5 && filters.bypass_fsr < 0.5) {
        // Sample 12 taps in a diamond pattern for edge reconstruction
        let p0 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 0.0, -1.0)).rgb;
        let p1 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 1.0, -1.0)).rgb;
        let p2 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(-1.0,  0.0)).rgb;
        let p3 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 0.0,  0.0)).rgb;
        let p4 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 1.0,  0.0)).rgb;
        let p5 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 2.0,  0.0)).rgb;
        let p6 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(-1.0,  1.0)).rgb;
        let p7 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 0.0,  1.0)).rgb;
        let p8 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 1.0,  1.0)).rgb;
        let p9 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 2.0,  1.0)).rgb;
        let p10= textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 0.0,  2.0)).rgb;
        let p11= textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>( 1.0,  2.0)).rgb;

        let w0 = get_brightness(p0); let w1 = get_brightness(p1);
        let w2 = get_brightness(p2); let w3 = get_brightness(p3);
        let w4 = get_brightness(p4); let w5 = get_brightness(p5);
        let w6 = get_brightness(p6); let w7 = get_brightness(p7);
        let w8 = get_brightness(p8); let w9 = get_brightness(p9);
        let w10= get_brightness(p10);let w11= get_brightness(p11);

        // Edge direction analysis
        let g_x = abs((w1 + w4 + w8 + w11) - (w0 + w3 + w7 + w10));
        let g_y = abs((w6 + w7 + w8 + w9) - (w2 + w3 + w4 + w5));
        
        // Advanced weight calculation based on gradient
        let ratio_x = 1.0 / (1.0 + g_x);
        let ratio_y = 1.0 / (1.0 + g_y);
        let norm_w = vec2<f32>(ratio_x, ratio_y) / (ratio_x + ratio_y);
        
        // Edge-aware reconstruction using central 4-taps with gradient bias
        rgb = mix(mix(p3, p4, norm_w.x), mix(p7, p8, norm_w.x), norm_w.y);
    } else {
        rgb = base_sampled.rgb;
    }

    // 2. Base Color Adjustments
    if (filters.bypass_color < 0.5) {
        // Apply gamut conversion matrix from image profile space to display profile space.
        rgb = filters.color_matrix * rgb;

        rgb = rgb * exp2(filters.exposure);
        rgb = (rgb - vec3<f32>(0.5)) * filters.contrast + vec3<f32>(0.5);
        rgb = rgb + vec3<f32>(filters.bright);
        rgb = max(rgb, vec3<f32>(0.0));
        rgb = pow(rgb, vec3<f32>(1.0 / max(filters.gamma, 0.001)));
    }
    rgb = pow(rgb, vec3<f32>(filters.icc_gamma));

    // 3. Median Filter
    if (filters.bypass_median < 0.5 && filters.median_enabled > 0.5) {
        var m: array<f32, 9>;
        var idx = 0;
        let stride = filters.median_stride;
        for (var y = -1; y <= 1; y++) {
            for (var x = -1; x <= 1; x++) {
                let c = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(f32(x) * stride, f32(y) * stride)).rgb;
                m[idx] = get_brightness(c);
                idx++;
            }
        }
        for (var i = 0; i < 5; i++) {
            for (var j = i + 1; j < 9; j++) {
                if (m[i] > m[j]) {
                    let tmp = m[i]; m[i] = m[j]; m[j] = tmp;
                }
            }
        }
        let luma = get_brightness(rgb);
        let median_luma = mix(luma, m[4], filters.median_strength);
        rgb = rgb * (median_luma / max(luma, 0.001));
    }

    // 4. Gaussian Blur / Unsharp Mask (Detail Section)
    if (filters.bypass_detail < 0.5 && (filters.blur_radius > 0.01 || filters.unsharp_amount > 0.01)) {
        let stride = filters.blur_radius + 1.0;
        let g0 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(0.0, 0.0)).rgb;
        let g1 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(-stride, 0.0)).rgb;
        let g2 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(stride, 0.0)).rgb;
        let g3 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(0.0, -stride)).rgb;
        let g4 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(0.0, stride)).rgb;
        
        let blurred = (g0 * 4.0 + g1 + g2 + g3 + g4) / 8.0;
        
        if (filters.unsharp_amount > 0.01) {
            let diff = rgb - blurred;
            if (get_brightness(abs(diff)) > filters.unsharp_threshold) {
                rgb = rgb + diff * filters.unsharp_amount;
            }
        } else {
            rgb = blurred;
        }
    }

    // 5. Levels
    if (filters.bypass_levels < 0.5) {
        let l_in_min = filters.levels_in_black;
        let l_in_max = max(filters.levels_in_white, l_in_min + 0.001);
        rgb = clamp((rgb - vec3<f32>(l_in_min)) / (l_in_max - l_in_min), vec3<f32>(0.0), vec3<f32>(1.0));
        rgb = pow(rgb, vec3<f32>(1.0 / max(filters.levels_gamma, 0.01)));
        rgb = mix(vec3<f32>(filters.levels_out_black), vec3<f32>(filters.levels_out_white), rgb);
    }

    // 6. AMD FSR 1.0 RCAS
    if (filters.fsr_enabled > 0.5 && filters.bypass_fsr < 0.5) {
        let c0 = rgb;
        let c1 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(-1.0, 0.0)).rgb;
        let c2 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(1.0, 0.0)).rgb;
        let c3 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(0.0, -1.0)).rgb;
        let c4 = textureSample(t_diffuse, s_diffuse, in.tex_coords + texel * vec2<f32>(0.0, 1.0)).rgb;
        
        let mn = min(c0, min(min(c1, c2), min(c3, c4)));
        let mx = max(c0, max(max(c1, c2), max(c3, c4)));
        
        let edge = c0 * 5.0 - (c1 + c2 + c3 + c4);
        let sharp = filters.fsr_sharpness * 0.5;
        rgb = clamp(c0 + edge * sharp, mn, mx);
    }

    return vec4<f32>(rgb, base_sampled.a);
}
