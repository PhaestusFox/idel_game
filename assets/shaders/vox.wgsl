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
    var pos = floor(mesh.world_position.xyz + mesh.world_normal * 0.5);
    let sample_pos = ((pos % chunk_size + lod) + chunk_size) % chunk_size;
    let c = textureSample(material_color_texture, material_color_sampler, sample_pos / (chunk_size * lod));
    if c.a < 0.2 {
        discard;
    } else {
        return c;
    }
}