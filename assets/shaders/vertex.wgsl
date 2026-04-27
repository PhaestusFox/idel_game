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

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

    let pos = resolve_pos(vertex_no_morph.position, chunk_size);

    // If chunk is standard
    if lod == 1 {
        var nx = 0u;
        var nz = 0u;
        var ny = 0u;
        if out.world_normal.y != 0. {
            if pos.x == 0 && pos.z == 0 {
                nx = chunk_size;
                ny = u32(pos.y);
                nz = chunk_size;
                for (var x = 0u; x < chunk_size; x++) {
                    for (var z = 0u; z < chunk_size; z++) {
                        let block = get_block(vec3u(x, ny, z));
                        if block != 0u {
                            nx = min(x, nx);
                            nz = min(z, nz);
                        }
                    }
                }
                if nx == chunk_size || nz == chunk_size {
                    out.position = vec4<f32>(0.0, 0., -1.0, 0.0);
                    return out;
                } else {
                    out.world_position.x += f32(nx);
                    out.world_position.z += f32(nz);
                }
            } else if pos.x == 0 && pos.z == chunk_size {
                nx = chunk_size;
                ny = u32(pos.y);
                nz = 0;
                for (var x = 0u; x < chunk_size; x++) {
                    for (var z = 0u; z < chunk_size; z++) {
                        let block = get_block(vec3u(x, ny, z));
                        if block != 0u {
                            nx = min(x, nx);
                            nz = max(z, nz);
                        }
                    }
                }
                out.world_position.x += f32(nx);
                out.world_position.z += f32(nz);
            }
        }
        if out.world_normal.y != 1. {
            out.position = vec4<f32>(0.0, 0., -1.0, 0.0);
        } else {
            out.position = position_world_to_clip(out.world_position.xyz);
        }
    }
    // If chunk is solid, only render faces that are on the edge of the chunk.
    else if lod == 1 << 31 {
        let x = out.world_normal.x != 0. && (pos.x == 0 || pos.x == u32(chunk_size));
        let y = out.world_normal.y != 0. && (pos.y == 0 || pos.y == u32(chunk_size));
        let z = out.world_normal.z != 0. && (pos.z == 0 || pos.z == u32(chunk_size));   
        if !(x || y || z) {
            out.position = vec4<f32>(0.0, 0., -1.0, 0.0);
        }
    }

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