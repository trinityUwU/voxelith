// Responsabilité : shader terrain — transforme par la view-proj caméra, échantillonne
// le texture array (layer + UV de tiling), applique teinte de biome + éclairage + fog.

struct Camera {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var atlas: texture_2d_array<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tint: vec3<f32>,
    @location(4) layer: u32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tint: vec3<f32>,
    @location(3) @interpolate(flat) layer: u32,
    @location(4) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    out.uv = in.uv;
    out.tint = in.tint;
    out.layer = in.layer;
    out.world_pos = in.position;
    return out;
}

const FOG_COLOR: vec3<f32> = vec3<f32>(0.62, 0.74, 0.92);

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let albedo = textureSample(atlas, atlas_sampler, in.uv, in.layer).rgb * in.tint;

    let light_dir = normalize(vec3<f32>(0.45, 1.0, 0.32));
    let diffuse = max(dot(normalize(in.normal), light_dir), 0.0);
    let lit = albedo * (0.42 + 0.58 * diffuse);

    // Fog de distance : ferme l'horizon, masque le bord du terrain chargé.
    let dist = length(in.world_pos - camera.camera_pos.xyz);
    let fog = clamp((dist - 370.0) / 110.0, 0.0, 1.0);
    return vec4<f32>(mix(lit, FOG_COLOR, fog), 1.0);
}
