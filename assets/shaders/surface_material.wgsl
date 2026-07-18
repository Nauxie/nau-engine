#import bevy_pbr::{
    mesh_view_bindings::globals,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_types::PbrInput,
    pbr_functions::{
        alpha_discard,
        apply_pbr_lighting,
        calculate_tbn_mikktspace,
        main_pass_post_lighting_processing,
    },
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    pbr_deferred_functions::deferred_output,
    prepass_io::{VertexOutput, FragmentOutput},
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

struct SurfaceMaterialUniform {
    primary_tint: vec4<f32>,
    secondary_tint: vec4<f32>,
    accent_tint: vec4<f32>,
    parameters: vec4<f32>,
    motion: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> surface: SurfaceMaterialUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var surface_detail_normal: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var surface_detail_sampler: sampler;

fn hash_2d(value: vec2<f32>) -> f32 {
    let projected = dot(value, vec2<f32>(127.1, 311.7));
    return fract(sin(projected) * 43758.5453123);
}

fn value_noise(value: vec2<f32>) -> f32 {
    let cell = floor(value);
    let local = fract(value);
    let smooth_local = local * local * (3.0 - 2.0 * local);
    let a = hash_2d(cell);
    let b = hash_2d(cell + vec2<f32>(1.0, 0.0));
    let c = hash_2d(cell + vec2<f32>(0.0, 1.0));
    let d = hash_2d(cell + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, smooth_local.x), mix(c, d, smooth_local.x), smooth_local.y);
}

fn terrain_fbm(position: vec2<f32>, phase: f32) -> vec3<f32> {
    let warp = vec2<f32>(
        value_noise(position * 0.61 + vec2<f32>(phase * 0.71, phase * 1.19)),
        value_noise(position * 0.61 + vec2<f32>(-phase * 1.07, phase * 0.43)),
    ) - 0.5;
    let warped = position + warp * 1.35;
    let rotated = vec2<f32>(
        warped.x * 0.74 + warped.y * 0.67,
        warped.y * 0.74 - warped.x * 0.67,
    );
    let macro_noise = value_noise(warped * 0.78 + vec2<f32>(phase, phase * 0.37));
    let middle = value_noise(rotated * 2.31 + vec2<f32>(phase * 1.71, -phase));
    let detail = value_noise(warped * 6.37 - vec2<f32>(phase * 0.43, phase * 1.13));
    let micro = value_noise(rotated * 15.73 + vec2<f32>(phase * 2.09, phase * 0.61));
    let detail_mix = detail * 0.68 + micro * 0.32;
    return vec3<f32>(macro_noise, middle, detail_mix);
}

fn tint_at_luma(tint: vec3<f32>, target_luma: f32) -> vec3<f32> {
    let tint_luma = max(dot(tint, vec3<f32>(0.2126, 0.7152, 0.0722)), 0.025);
    return clamp(tint * target_luma / tint_luma, vec3<f32>(0.0), vec3<f32>(1.25));
}

fn sampled_wave_normal(uv: vec2<f32>, time: f32, flow: f32) -> vec3<f32> {
    let scale = surface.motion.x;
    let speed = surface.motion.z;
    let phase = surface.parameters.y;
    let calm_motion = vec2<f32>(0.037, 0.021);
    let flow_motion = vec2<f32>(0.012, -0.110);
    let primary_motion = mix(calm_motion, flow_motion, flow);
    let secondary_motion = mix(vec2<f32>(-0.021, 0.031), vec2<f32>(-0.018, -0.074), flow);
    let tertiary_motion = mix(vec2<f32>(0.016, -0.025), vec2<f32>(0.026, -0.138), flow);
    let quaternary_motion = mix(vec2<f32>(-0.034, -0.012), vec2<f32>(-0.012, -0.190), flow);

    let uv0 = uv * scale + primary_motion * time * speed + phase;
    let uv1 = uv * scale * 2.13 + secondary_motion * time * speed - phase * 0.71;
    let uv2 = uv * scale * 4.27 + tertiary_motion * time * speed + phase * 1.37;
    let uv3 = uv * scale * 8.41 + quaternary_motion * time * speed - phase * 1.91;
    let n0 = textureSample(surface_detail_normal, surface_detail_sampler, uv0).xyz * 2.0 - 1.0;
    let n1 = textureSample(surface_detail_normal, surface_detail_sampler, uv1).xyz * 2.0 - 1.0;
    let n2 = textureSample(surface_detail_normal, surface_detail_sampler, uv2).xyz * 2.0 - 1.0;
    let n3 = textureSample(surface_detail_normal, surface_detail_sampler, uv3).xyz * 2.0 - 1.0;
    let slope = n0.xy * 0.44 + n1.xy * 0.29 + n2.xy * 0.18 + n3.xy * 0.09;
    return normalize(vec3<f32>(slope * surface.motion.y, 1.0));
}

fn shade_terrain(in: VertexOutput, pbr_input: PbrInput) -> PbrInput {
    var result = pbr_input;
    let world_xz = result.world_position.xz;
    let noise = terrain_fbm(
        world_xz * surface.parameters.z,
        surface.parameters.y,
    );
    var highland = 0.0;
    var exposed_edge = 0.0;
#ifdef VERTEX_UVS_B
    highland = clamp(in.uv_b.x, 0.0, 1.0);
    exposed_edge = clamp(in.uv_b.y, 0.0, 1.0);
#else
    highland = smoothstep(0.35, 0.78, result.world_normal.y);
    exposed_edge = 1.0 - smoothstep(0.48, 0.86, result.world_normal.y);
#endif

    let macro_wash = (noise.x - 0.5) * 0.24;
    let middle_wash = (noise.y - 0.5) * 0.34;
    let detail_wash = (noise.z - 0.5) * 0.16 * surface.parameters.w;
    let ridge_wash = smoothstep(0.08, 0.34, abs(noise.x - noise.y)) * 0.12 - 0.04;
    let palette_mix = clamp(highland * 0.48 + noise.y * 0.30, 0.0, 0.78);
    let dry_dapple = smoothstep(0.56, 0.84, noise.x)
        * 0.36
        * (1.0 - exposed_edge * 0.35);
    let edge_mix = clamp(
        exposed_edge * (0.46 + noise.x * 0.30) + dry_dapple,
        0.0,
        0.84,
    );
    var terrain_color = result.material.base_color.rgb;
    let base_luma = max(dot(terrain_color, vec3<f32>(0.2126, 0.7152, 0.0722)), 0.035);
    let secondary_color = tint_at_luma(
        surface.secondary_tint.rgb,
        base_luma * (0.80 + noise.y * 0.20),
    );
    let accent_color = tint_at_luma(
        surface.accent_tint.rgb,
        base_luma * (1.04 + noise.x * 0.30),
    );
    terrain_color = mix(
        terrain_color,
        secondary_color,
        palette_mix * 0.84,
    );
    terrain_color = mix(
        terrain_color,
        accent_color,
        edge_mix * 0.84,
    );
    terrain_color *= 1.01 + macro_wash + middle_wash + detail_wash + ridge_wash;
    terrain_color += surface.primary_tint.rgb * (noise.x - noise.y) * 0.050;
    result.material.base_color = vec4<f32>(
        max(terrain_color, vec3<f32>(0.015)),
        result.material.base_color.a,
    );
    let terrain_shadow_fill =
        0.060 + noise.z * 0.080 + abs(noise.x - noise.y) * 0.055;
    result.material.emissive = vec4<f32>(
        result.material.emissive.rgb + terrain_color * terrain_shadow_fill,
        result.material.emissive.a,
    );
    result.material.perceptual_roughness = clamp(
        result.material.perceptual_roughness
            + (noise.y - 0.5) * 0.13
            - exposed_edge * 0.08,
        0.46,
        1.0,
    );
    result.diffuse_occlusion *= 0.91 + noise.x * 0.09;
    return result;
}

fn shade_water(in: VertexOutput, pbr_input: PbrInput, is_front: bool) -> PbrInput {
    var result = pbr_input;
    var foam_mask = 0.0;
    var flow = 0.0;
#ifdef VERTEX_UVS_B
    foam_mask = clamp(in.uv_b.x, 0.0, 1.0);
    flow = clamp(in.uv_b.y, 0.0, 1.0);
#endif
    let waterfall = smoothstep(0.82, 0.98, flow);
    var uv = result.world_position.xz * surface.parameters.z;
#ifdef VERTEX_UVS_A
    uv = in.uv;
#endif

    let detail_normal = sampled_wave_normal(uv, globals.time, flow);
    let face_sign = select(-1.0, 1.0, is_front);
#ifdef VERTEX_TANGENTS
    let tangent_frame = calculate_tbn_mikktspace(in.world_normal, in.world_tangent);
    result.N = normalize(tangent_frame * (detail_normal * face_sign));
#else
    let wave_blend = surface.motion.y * (1.0 - flow * 0.18);
    result.N = normalize(mix(
        result.N,
        normalize(vec3<f32>(detail_normal.x, 1.0, detail_normal.y)) * face_sign,
        wave_blend,
    ));
#endif

    let facing = clamp(dot(result.N, result.V), 0.0, 1.0);
    let fresnel = pow(1.0 - facing, 4.0);
    let wave_glint = pow(max(detail_normal.z, 0.0), 5.0);
    let flow_crest = smoothstep(0.68, 0.98, value_noise(
        vec2<f32>(uv.x * 7.0, uv.y * mix(3.0, 13.0, flow))
            + vec2<f32>(surface.parameters.y, -globals.time * surface.motion.z * (2.0 + flow * 5.0)),
    ));
    let flow_breakup = value_noise(
        vec2<f32>(uv.x * 11.0, uv.y * 4.0)
            + vec2<f32>(surface.parameters.y * 0.37, -globals.time * surface.motion.z * 4.0),
    );
    let basin_breakup = value_noise(
        uv * 6.2
            + vec2<f32>(
                surface.parameters.y + globals.time * surface.motion.z * 0.07,
                -surface.parameters.y * 0.61 - globals.time * surface.motion.z * 0.04,
            ),
    );
    let crossing_breakup = value_noise(
        vec2<f32>(uv.x * 12.4 + uv.y * 3.1, uv.y * 10.8 - uv.x * 2.7)
            + vec2<f32>(-surface.parameters.y * 0.29, globals.time * surface.motion.z * 0.11),
    );
    let stream = smoothstep(0.18, 0.72, flow);
    let lateral_edge = smoothstep(0.0, 0.115, min(uv.x, 1.0 - uv.x));
    let falling_streaks = 0.58 + flow_breakup * 0.42;
    let silhouette_alpha = mix(1.0, lateral_edge * falling_streaks, waterfall);
    let foam = clamp(
        foam_mask * surface.motion.w * (0.48 + flow_crest * 0.72)
            + flow * flow_crest * 0.18,
        0.0,
        1.0,
    );
    let depth_mix = clamp(0.34 + fresnel * 0.56 + wave_glint * 0.10, 0.0, 1.0);
    var water_color = mix(surface.primary_tint.rgb, surface.secondary_tint.rgb, depth_mix);
    water_color = mix(water_color, surface.accent_tint.rgb, foam);
    let falling_color = mix(
        surface.secondary_tint.rgb,
        surface.accent_tint.rgb,
        0.28 + foam * 0.52,
    );
    water_color = mix(water_color, falling_color, waterfall * 0.46);
    let basin_crest = smoothstep(0.62, 0.86, basin_breakup) * (1.0 - stream);
    water_color = mix(
        water_color,
        surface.accent_tint.rgb,
        clamp(basin_crest * 0.12 + wave_glint * 0.045 * (1.0 - stream), 0.0, 0.18),
    );
    let basin_shimmer =
        0.80 + basin_breakup * 0.22 + crossing_breakup * 0.12 + wave_glint * 0.08;
    let flow_shimmer = mix(basin_shimmer, 0.86 + flow_breakup * 0.28, stream);
    water_color *= flow_shimmer;
    result.material.base_color = vec4<f32>(
        water_color,
        result.material.base_color.a
            * mix(0.92 + fresnel * 0.08, 0.70 + flow_breakup * 0.25, stream)
            * (0.90 + foam * 0.10)
            * silhouette_alpha,
    );
    result.material.perceptual_roughness = mix(
        mix(0.14, 0.36, foam),
        0.30,
        waterfall * 0.45,
    );
    let water_shadow_fill =
        0.18 + basin_breakup * 0.26 + crossing_breakup * 0.14 + wave_glint * 0.05;
    result.material.emissive = vec4<f32>(
        result.material.emissive.rgb
            + water_color * (water_shadow_fill + waterfall * 0.035)
            + surface.accent_tint.rgb * foam * 0.065,
        result.material.emissive.a,
    );
    return result;
}

fn shade_foam(in: VertexOutput, pbr_input: PbrInput) -> PbrInput {
    var result = pbr_input;
    var uv = result.world_position.xz * surface.parameters.z;
#ifdef VERTEX_UVS_A
    uv = in.uv;
#endif
    let moving_noise = value_noise(
        uv * surface.motion.x
            + vec2<f32>(
                globals.time * surface.motion.z,
                -globals.time * surface.motion.z * 0.63,
            ),
    );
    let broken_edge = smoothstep(0.28, 0.78, moving_noise);
    let foam_color = mix(
        surface.primary_tint.rgb,
        surface.accent_tint.rgb,
        broken_edge,
    );
    result.material.base_color = vec4<f32>(
        foam_color,
        result.material.base_color.a * (0.58 + broken_edge * 0.37),
    );
    result.material.perceptual_roughness = 0.44 + (1.0 - broken_edge) * 0.18;
    return result;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let surface_kind = surface.parameters.x;
    if surface_kind < 0.5 {
        pbr_input = shade_terrain(in, pbr_input);
    } else if surface_kind < 1.5 {
        pbr_input = shade_water(in, pbr_input, is_front);
    } else {
        pbr_input = shade_foam(in, pbr_input);
    }
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material,
        pbr_input.material.base_color,
    );

#ifdef PREPASS_PIPELINE
    return deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
#endif
}
