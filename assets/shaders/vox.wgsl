const ERROR_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 1.0,1.);

@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> lod: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> chunk_offset: vec3<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> data: array<vec4<u32>,  (16 * 16 * 16) / 4>;

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

const chunk_size: f32 = 16.;


@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    /// calc chunk id for world position
    /// calc chunk offset
    /// sub offset from world position to get local position
    /// workout what voxel you are
    /// shift by 0.5 away from edge so the sampler takes from middle of the voxel not the edge and wraps
    var pos = mesh.world_position.xyz - chunk_offset;
    var offset = sign(mesh.world_normal) * 0.5;
    pos = floor(pos - offset);
    // var sample_pos = (((pos % chunk_size) + chunk_size) % chunk_size);
    var sample_pos = pos;
    sample_pos /= lod;
    sample_pos = floor(sample_pos) * lod + lod * 0.5;
    if pos.y >= chunk_size {
        discard;
    };
    if pos.x >= chunk_size {
        discard;
    };
    if pos.z >= chunk_size{
        discard;
    };
    if pos.x <= -1.0 {
        discard;
    };
    if pos.y <= -1.0 {
        discard;
    };
    if pos.z <= -1.0 {
        discard;
    };
    sample_pos /= chunk_size;
    // sample_pos = clamp(sample_pos, vec3(0.), vec3(chunk_size));
    // var c = textureSample(material_color_texture, material_color_sampler, sample_pos);
    let x = u32(pos.x);
    let index = (u32(pos.z) * u32(chunk_size) * u32(chunk_size) + u32(pos.y) * u32(chunk_size) + x) / 4u;
    let block_data = data[index];
    var block_id: u32 = 0;
    switch (x % 4u) {
        case 0u: {
            block_id = block_data.x;
        }
        case 1: {
            block_id = block_data.y;
        }
        case 2: {
            block_id = block_data.z;
        }
        case 3: {
            block_id = block_data.w;
        }
        default: {
            block_id = block_data.x;
        }
    };
    if block_id == 0u {
        discard;
    }
    let c = textureSample(material_color_texture, material_color_sampler, vec2(f32(block_id) / 255., 0.5));


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