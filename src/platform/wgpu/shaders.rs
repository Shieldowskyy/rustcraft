//! WGSL shader sources for WGPU terrain/sky passes and compute framebuffer readback.

pub(super) const SHADER: &str = r#"
struct Globals {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_position: vec4<f32>,
    camera_pitch: vec4<f32>,
    sky_top: vec4<f32>,
    sky_bot: vec4<f32>,
    fog_color: vec4<f32>,
    sun_dir: vec4<f32>,
    moon_dir: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var textures: texture_2d_array<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) light: vec4<f32>,
    @location(3) texture_layer: u32,
    @location(4) y_offset: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) texture_layer: u32,
    @location(2) light_color: vec3<f32>,
    @location(3) world_position: vec3<f32>,
};

@vertex
fn world_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pos = vec3<f32>(input.position.x, input.position.y + input.y_offset, input.position.z);
    let clip = globals.view_proj * vec4<f32>(pos, 1.0);
    output.position = vec4<f32>(clip.x, clip.y, clip.z * 0.5 + clip.w * 0.5, clip.w);
    output.uv = input.uv;
    output.texture_layer = input.texture_layer;
    output.world_position = pos;

    let daylight = globals.camera_pitch.w;
    let sky_ambient = mix(vec3<f32>(0.05, 0.07, 0.14), globals.sky_bot.xyz, clamp(daylight * 1.1, 0.0, 1.0));
    let sun_color = mix(vec3<f32>(0.14, 0.18, 0.28), vec3<f32>(1.06, 0.98, 0.88), clamp(daylight, 0.0, 1.0));
    let sky_factor = pow(input.light.y, 1.25);
    let natural_light = (sun_color * 0.85 + sky_ambient * 0.35) * (sky_factor * daylight);

    let torch_color = vec3<f32>(1.30, 0.88, 0.42);
    let block_factor = pow(input.light.z, 1.15);
    let torch_light = torch_color * block_factor;

    let cave_ambient = vec3<f32>(0.045, 0.045, 0.055);

    let total_light = (natural_light + torch_light + cave_ambient) * input.light.x;
    output.light_color = clamp(total_light, vec3<f32>(0.04), vec3<f32>(1.35));
    return output;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) linear_depth: f32,
};

@fragment
fn world_fragment(input: VertexOutput) -> FragmentOutput {
    let texel = textureSample(textures, texture_sampler, input.uv, i32(input.texture_layer));
    let distance_to_camera = distance(input.world_position, globals.camera_position.xyz);
    let lit = texel.rgb * input.light_color;
    let max_fog_dist = select(globals.camera_pitch.z, 180.0, globals.camera_pitch.z <= 0.0);
    let fog_t = clamp((distance_to_camera - max_fog_dist * 0.45) / (max_fog_dist * 0.55), 0.0, 1.0) * globals.camera_pitch.y;
    let final_rgb = mix(lit, globals.fog_color.xyz, fog_t);
    return FragmentOutput(vec4<f32>(final_rgb, 1.0), distance_to_camera);
}

@fragment
fn world_fragment_cutout(input: VertexOutput) -> FragmentOutput {
    let texel = textureSample(textures, texture_sampler, input.uv, i32(input.texture_layer));
    let distance_to_camera = distance(input.world_position, globals.camera_position.xyz);
    if (texel.a < 0.1) {
        if (distance_to_camera < 120.0) {
            discard;
        }
    }
    let lit = texel.rgb * input.light_color;
    let max_fog_dist = select(globals.camera_pitch.z, 180.0, globals.camera_pitch.z <= 0.0);
    let fog_t = clamp((distance_to_camera - max_fog_dist * 0.45) / (max_fog_dist * 0.55), 0.0, 1.0) * globals.camera_pitch.y;
    let final_rgb = mix(lit, globals.fog_color.xyz, fog_t);
    return FragmentOutput(vec4<f32>(final_rgb, 1.0), distance_to_camera);
}

@fragment
fn world_fragment_trans(input: VertexOutput) -> FragmentOutput {
    let texel = textureSample(textures, texture_sampler, input.uv, i32(input.texture_layer));
    let distance_to_camera = distance(input.world_position, globals.camera_position.xyz);
    let lit = texel.rgb * input.light_color;
    let max_fog_dist = select(globals.camera_pitch.z, 180.0, globals.camera_pitch.z <= 0.0);
    let fog_t = clamp((distance_to_camera - max_fog_dist * 0.45) / (max_fog_dist * 0.55), 0.0, 1.0) * globals.camera_pitch.y;
    let final_rgb = mix(lit, globals.fog_color.xyz, fog_t);
    let alpha = select(1.0, 0.48, input.texture_layer == 8u || input.texture_layer == 19u);
    return FragmentOutput(vec4<f32>(final_rgb, alpha), distance_to_camera);
}

struct SkyOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn sky_vertex(@builtin(vertex_index) index: u32) -> SkyOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0)
    );
    var output: SkyOutput;
    let position = positions[index];
    output.position = vec4<f32>(position, 1.0, 1.0);
    output.ndc = position;
    return output;
}

struct SkyFragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) linear_depth: f32,
};

@fragment
fn sky_fragment(input: SkyOutput) -> SkyFragmentOutput {
    let clip = vec4<f32>(input.ndc.x, input.ndc.y, 1.0, 1.0);
    let world_h = globals.inv_view_proj * clip;
    let world_pos = world_h.xyz / world_h.w;
    let ray_dir = normalize(world_pos - globals.camera_position.xyz);

    let t = clamp(ray_dir.y * 0.8 + 0.2, 0.0, 1.0);
    var col = mix(globals.sky_bot.xyz, globals.sky_top.xyz, t);

    let sun_dir = normalize(globals.sun_dir.xyz);
    let sun_dot = dot(ray_dir, sun_dir);
    if (sun_dot > 0.993) {
        let sun_disc = smoothstep(0.993, 0.997, sun_dot);
        col = mix(col, vec3<f32>(1.0, 0.98, 0.75), sun_disc);
    }
    if (sun_dot > 0.85) {
        let sun_glow = pow(sun_dot, 32.0) * 0.45;
        col += vec3<f32>(1.0, 0.65, 0.25) * sun_glow;
    }

    let moon_dir = normalize(globals.moon_dir.xyz);
    let moon_dot = dot(ray_dir, moon_dir);
    if (moon_dot > 0.995) {
        let moon_disc = smoothstep(0.995, 0.998, moon_dot);
        col = mix(col, vec3<f32>(0.92, 0.95, 1.0), moon_disc);
    }
    if (moon_dot > 0.90) {
        let moon_glow = pow(moon_dot, 40.0) * 0.25;
        col += vec3<f32>(0.5, 0.65, 0.95) * moon_glow;
    }

    let daylight = globals.camera_pitch.w;
    if (daylight < 0.65 && ray_dir.y > 0.02) {
        let star_alpha = clamp((0.65 - daylight) / 0.50, 0.0, 1.0);
        let star_grid = floor(ray_dir * 160.0);
        let star_rand = fract(sin(dot(star_grid, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
        if (star_rand > 0.988) {
            let star_bright = (star_rand - 0.988) / 0.012 * star_alpha;
            col += vec3<f32>(star_bright * 0.9, star_bright * 0.95, star_bright * 1.0);
        }
    }

    return SkyFragmentOutput(vec4<f32>(col, 1.0), bitcast<f32>(0x7F800000u));
}
"#;

pub(super) const POST_PROCESS_SHADER: &str = r#"
@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> out_color: array<u32>;
@group(0) @binding(3) var<storage, read_write> out_depth: array<f32>;

@compute @workgroup_size(16, 16)
fn post_process(@builtin(global_invocation_id) id: vec3<u32>) {
    let dim = textureDimensions(color_tex);
    if (id.x >= dim.x || id.y >= dim.y) {
        return;
    }
    let idx = id.y * dim.x + id.x;

    let color = textureLoad(color_tex, id.xy, 0);
    let r = u32(color.r * 255.0);
    let g = u32(color.g * 255.0);
    let b = u32(color.b * 255.0);
    out_color[idx] = (r << 16u) | (g << 8u) | b;

    let d = textureLoad(depth_tex, id.xy, 0).r;
    out_depth[idx] = d;
}
"#;
