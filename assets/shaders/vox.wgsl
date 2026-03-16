const ERROR_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 1.0,1.);

@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> lod: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> chunk_offset: vec3<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var myTexture: texture_storage_3d<rgba8uint, read>;
// @group(#{MATERIAL_BIND_GROUP}) @binding(6) var data_sampler: sampler;

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
		let pos = resolve_pos(mesh);
		let block = get_block(pos);
		let block_id = block & 0x000000FFu;
    if block_id == 0u {
        discard;
    }

    let c = textureSample(material_color_texture, material_color_sampler, vec2(f32(block_id) / 255. + (1 / 512.), 0.5));
	if c.a < 0.2 { discard; }

    let l: f32 = shading_of_normal(mesh.world_normal);
    return c * vec4(vec3(l), 1.);
}

fn resolve_pos(mesh: VertexOutput) -> vec3u {
    var pos = floor(mesh.world_position.xyz - chunk_offset - mesh.world_normal.xyz*0.5);
		pos = clamp(pos, vec3(0), vec3(chunk_size-1));
		pos = (pos % chunk_size + chunk_size) % chunk_size;
		return vec3u(pos);
}

fn get_block(pos: vec3u) -> u32 {
    let index = (pos.z * u32(chunk_size) * u32(chunk_size) + pos.y * u32(chunk_size) + pos.x);
    let block_data = textureLoad(myTexture, pos);
    return block_data.x;
    // switch (pos.x % 4u) {
    //     case 0u: {
    //         return block_data.x;
    //     }
    //     case 1u: {
    //         return block_data.y;
    //     }
    //     case 2u: {
    //         return block_data.z;
    //     }
    //     case 3u: {
    //         return block_data.w;
    //     }
    //     default: {
    //         return 255u;
    //     }
    // };
}

fn shading_of_normal(normal: vec3f) -> f32 {
    if (normal.x == 0.0 && normal.y == 0.0 && normal.z == 0.0) {
        return 0.0; // default
    }
    else if normal.y > 0.5 {
        return 0.75; // bottom
    } else if normal.y < -0.5 {
        return 1; // top
    } else if normal.x > 0.5 {
        return 0.85; // right
    } else if normal.x < -0.5 {
        return 0.95; // left
    } else if normal.z > 0.5 {
        return 0.90; // front
    } else {
        return 0.80; // back
    }
}
