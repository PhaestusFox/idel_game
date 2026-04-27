@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> lod: u32;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var myTexture: texture_storage_3d<rgba8uint, read>;

const chunk_size: u32 = 64;
const recp_chunk_size: f32 = 1.0 / f32(chunk_size);

#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
    view_bindings, pbr_functions,
};

#ifdef MORPH_TARGETS
// The instance_index parameter must match vertex_in.instance_index. This is a work around for a wgpu dx12 bug.
// See https://github.com/gfx-rs/naga/issues/2416
fn morph_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;

    let weight_count = bevy_pbr::morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i, instance_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, instance_index);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph_normal(vertex_index, i, instance_index);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, instance_index), 0.0);
#endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph, vertex_no_morph.instance_index);
#else
    var vertex = vertex_no_morph;
#endif

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

#ifdef SKINNED
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#else
    var world_from_local = mesh_world_from_local;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif


let pos = resolve_pos(vertex_no_morph.position, chunk_size);

// If chunk is solid, only render faces that are on the edge of the chunk.
if lod == 0xFFFFFFFF {
    let x = out.world_normal.x != 0. && (pos.x == 0 || pos.x == u32(chunk_size));
    let y = out.world_normal.y != 0. && (pos.y == 0 || pos.y == u32(chunk_size));
    let z = out.world_normal.z != 0. && (pos.z == 0 || pos.z == u32(chunk_size));   
    if !(x || y || z) {
        out.position = vec4<f32>(0.0, 0., -1.0, 0.0);
        return out;
    }
} else {
    // If chunk is standard
    let min_x = (lod & 0x1F);
    let max_x = ((lod >> 15) & 0x1F << 1);
    let min_y = ((lod >> 5) & 0x1F);
    let max_y = ((lod >> 20) & 0x1F) << 1;
    let min_z = ((lod >> 10) & 0x1F);
    let max_z = ((lod >> 25) & 0x1F) << 1;
    
    let x = out.world_normal.x != 0.;
    let y = out.world_normal.y != 0.;
    let z = out.world_normal.z != 0.;
    if x && (pos.x < (min_x + 1) || pos.x > (max_x + 1))
    || y && (pos.y < (min_y + 1) || pos.y > (max_y + 1))
    || z && (pos.z < (min_z + 1) || pos.z > (max_z + 1)) {
        // out.position = vec4<f32>(0.0, 0., -1.0, 0.0);
        // return out;
    }
    if x {
        if pos.y == 0 {
           vertex.position.y = f32(min_y) * recp_chunk_size * 2.0 - 1.0;
        } else {
            vertex.position.y = f32(max_y) * recp_chunk_size * 2.0 - 1.0;
        }
        if pos.z == 0 {
            vertex.position.z = f32(min_z) * recp_chunk_size * 2.0 - 1.0;
        } else {
            vertex.position.z = f32(max_z) * recp_chunk_size * 2.0 - 1.0;
        }
    } else if y {
        if pos.x == 0 {
           vertex.position.x = f32(min_x) * recp_chunk_size * 2.0 - 1.0;
        } else {
            vertex.position.x = f32(max_x) * recp_chunk_size * 2.0 - 1.0;
        }
        if pos.z == 0 {
            vertex.position.z = f32(min_z) * recp_chunk_size * 2.0 - 1.0;
        } else {
            vertex.position.z = f32(max_z) * recp_chunk_size * 2.0 - 1.0;
        }
    } else if z {
        if pos.x == 0 {
           vertex.position.x = f32(min_x) * recp_chunk_size * 2.0 - 1.0;
        } else {
            vertex.position.x = f32(max_x) * recp_chunk_size * 2.0 - 1.0;
        }
        if pos.y == 0 {
            vertex.position.y = f32(min_y) * recp_chunk_size * 2.0 - 1.0;
        } else {
            vertex.position.y = f32(max_y) * recp_chunk_size * 2.0 - 1.0;
        }
    }
}
out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
out.position = position_world_to_clip(out.world_position.xyz);

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}


fn resolve_pos(position: vec3<f32>, chunk_size: u32) -> vec3u {
    return vec3u((position + 1.0) * f32(chunk_size) * 0.5);
}

fn get_block(pos: vec3u) -> u32 {
    let index = (pos.z * chunk_size * chunk_size + pos.y * chunk_size + pos.x);
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