const ERROR_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 1.0,1.);

@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_texture: texture_3d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> lod: f32;

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
}

const chunk_size: f32 = 32.;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var offset = sign(mesh.world_normal);
    var pos = floor(mesh.world_position.xyz - mesh.world_normal * 0.5);
    var sample_pos = ((pos % chunk_size) + chunk_size) % chunk_size;
    sample_pos /= lod;
    sample_pos = floor(sample_pos) * lod + lod * 0.5 - offset * 0.5;
    sample_pos = clamp(sample_pos, vec3(0.), vec3(chunk_size));
    let c = textureSample(material_color_texture, material_color_sampler, sample_pos / chunk_size);
    var l: f32 = 0.;
    

    if mesh.world_normal.y > 0.5 {
        l = 0.75; // bottom
    } else if mesh.world_normal.y < -0.5 {
        l = 1; // top
    } else if mesh.world_normal.x > 0.5 {
        l = 0.85; // right
    } else if mesh.world_normal.x < -0.5 {
        l = 0.95; // left
    } else if mesh.world_normal.z > 0.5 {
        l = 0.90; // front
    } else {
        l = 0.80; // back
    }

    if c.a < 0.2 {
        discard;
    } else {
        return c * vec4(vec3(l), 1.);
    }
}