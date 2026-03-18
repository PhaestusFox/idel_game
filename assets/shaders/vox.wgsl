const ERROR_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 1.0,1.);
#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    view_bindings, pbr_functions};
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> lod: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var myTexture: texture_storage_3d<rgba8uint, read>;

@group(0) @binding(1) var<uniform> lights: Lights;

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

const chunk_size: f32 = 64.;

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

		let l = apply_light(c, mesh.world_position, mesh.world_normal);
        let face = shading_of_normal(mesh.world_normal);
		return l * face;
    //let l = (add_light(mesh.world_normal, mesh.world_position) / 10.) + vec3(0.1); // add some ambient lighting so that unlit faces aren't pitch black
    //return c * vec4(clamp(l.x, 0.0, 1.0), clamp(l.y, 0.0, 1.0), clamp(l.z, 0.0, 1.0), 1.0);
}

fn resolve_pos(mesh: VertexOutput) -> vec3u {
    let world_from_local: mat4x4<f32> = mesh_functions::get_world_from_local(mesh.instance_index);
    let offset = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(0., 0., 0., 1.0));
    var pos = floor(mesh.world_position.xyz - (offset.xyz + vec3(-chunk_size * 0.5)) - mesh.world_normal.xyz*0.5);
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
        return 0.90; // bottom
    } else if normal.y < -0.5 {
        return 1; // top
    } else if normal.x > 0.5 {
        return 0.94; // right
    } else if normal.x < -0.5 {
        return 0.98; // left
    } else if normal.z > 0.5 {
        return 0.96; // front
    } else {
        return 0.92; // back
    }
}

// //////////////////////// Here be dragons ////////////////////////
// /// This code is just bevy types I am pasting so my ide can see them
// /// 
struct Lights {
    // NOTE: this array size must be kept in sync with the constants defined in bevy_pbr/src/render/light.rs
    directional_lights: array<DirectionalLight, #{MAX_DIRECTIONAL_LIGHTS}u>,
    ambient_color: vec4<f32>,
    // x/y/z dimensions and n_clusters in w
    cluster_dimensions: vec4<u32>,
    // xy are vec2<f32>(cluster_dimensions.xy) / vec2<f32>(view.width, view.height)
    //
    // For perspective projections:
    // z is (cluster_dimensions.z - 1) / log(far / near)
    // w is (cluster_dimensions.z - 1) * log(near) / log(far / near)
    //
    // For orthographic projections:
    // NOTE: near and far are +ve but -z is infront of the camera
    // z is -near
    // w is cluster_dimensions.z / (-far - -near)
    cluster_factors: vec4<f32>,
    n_directional_lights: u32,
    spot_light_shadowmap_offset: i32,
    ambient_light_affects_lightmapped_meshes: u32
};

struct DirectionalLight {
    cascades: array<DirectionalCascade, #{MAX_CASCADES_PER_LIGHT}>,
    color: vec4<f32>,
    direction_to_light: vec3<f32>,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    soft_shadow_size: f32,
    shadow_depth_bias: f32,
    shadow_normal_bias: f32,
    num_cascades: u32,
    cascades_overlap_proportion: f32,
    depth_texture_base_index: u32,
    decal_index: u32,
    sun_disk_angular_size: f32,
    sun_disk_intensity: f32,
};

struct DirectionalCascade {
    clip_from_world: mat4x4<f32>,
    texel_size: f32,
    far_bound: f32,
}


const LAYER_BASE: u32 = 0;
const LAYER_CLEARCOAT: u32 = 1;

const AMBIENT_LIGHT: f32 = 0.1;

fn apply_light(color: vec4f, position:vec4f, normal: vec3f) -> vec4f {
  let view_dir = pbr_functions::calculate_view(position, false);

  var light = AMBIENT_LIGHT * (1 - max(dot(normal, view_dir), 0));

	for(var i = 0u; i < lights.n_directional_lights; i++){
		let curr = lights.directional_lights[i];

		let top = max(dot(vec3(0,1,0), curr.direction_to_light), 0);
        let pow = 1 / curr.color.r;
		if(top > 0) {
			let R = reflect(view_dir, normal);
			let direct = dot(R, curr.direction_to_light);
			if(direct > 0) {
				light += direct * top * pow;
			}

			let passive = dot(normal, curr.direction_to_light);
			if(passive > 0) {
				light += passive * top;
			}
		}
	}

	return color * clamp(light, 0.01, 1.);
}

fn add_light(world_normal: vec3<f32>, world_position: vec4<f32>) -> vec3f {
    let view_normal = pbr_functions::calculate_view(world_position, false);
    // let view_normal = vec3(0.);
    let NdotV = dot(world_normal, view_normal);
    let R = reflect(view_normal, world_normal);

    var lighting_input: LightingInput;
    lighting_input.layers[LAYER_BASE].NdotV = NdotV;
    lighting_input.layers[LAYER_BASE].N = world_normal;
    lighting_input.layers[LAYER_BASE].R = R;
    lighting_input.layers[LAYER_BASE].perceptual_roughness = 0.;
    lighting_input.layers[LAYER_BASE].roughness = 0.;
    lighting_input.P = world_position.xyz;
    lighting_input.V =  view_normal;
    // lighting_input.diffuse_color = vec3(1.);
    // lighting_input.metallic = 0.;
    // lighting_input.F0_dielectric = calculate_F0_dielectric(reflectance);
    // lighting_input.F0_metallic = output_color.rgb;
    // lighting_input.F_ab = F_ab;
#ifdef STANDARD_MATERIAL_ANISOTROPY
    lighting_input.anisotropy = 1.;
    lighting_input.Ta = vec3(1., 0., 0.);
    lighting_input.Ba = cross(world_normal, lighting_input.Ta); 
#endif  // STANDARD_MATERIAL_ANISOTROPY
    var direct_light: vec3<f32> = vec3<f32>(0.0);
    // directional lights (direct)
    let n_directional_lights = lights.n_directional_lights;
    for (var i: u32 = 0u; i < n_directional_lights; i = i + 1u) {
    // check if this light should be skipped, which occurs if this light does not intersect with the view
    // note point and spot lights aren't skippable, as the relevant lights are filtered in `assign_lights_to_clusters`
    let light = &lights.directional_lights[i];
    
    let enable_diffuse = true;
    
    var shadow: f32 = 1.0;
    // if ((in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u
    // && (view_bindings::lights.directional_lights[i].flags & mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u) {
    //     shadow = shadows::fetch_directional_shadow(i, in.world_position, in.world_normal, view_z, in.frag_coord.xy);
    // }
    
    // #ifdef DEPTH_PREPASS
    // if contact_shadow_enabled && (in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u && shadow > 0.0 &&
    // (view_bindings::lights.directional_lights[i].flags &
    //     mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_CONTACT_SHADOWS_ENABLED_BIT) != 0u {
    //         let L = view_bindings::lights.directional_lights[i].direction_to_light;
    //         shadow *= calculate_contact_shadow(in.world_position.xyz, in.frag_coord.xy, L, contact_shadow_steps);
    //     }
    //     #endif
        
    var light_contrib = directional_light(i, &lighting_input, enable_diffuse);
        
    direct_light += light_contrib * shadow;
    }
    

    return direct_light;
}

fn directional_light(
    light_id: u32,
    input: ptr<function, LightingInput>,
    enable_diffuse: bool
) -> vec3<f32> {
    // Unpack.
    let diffuse_color = vec3(1.);
    let NdotV = (*input).layers[LAYER_BASE].NdotV;
    let N = (*input).layers[LAYER_BASE].N;
    let V = (*input).V;
    let roughness = (*input).layers[LAYER_BASE].roughness;

    let light = &lights.directional_lights[light_id];

    let L = (*light).direction_to_light.xyz;
    var derived_input = derive_lighting_input(N, V, L);

    var diffuse = vec3(0.0);
    if (enable_diffuse) {
        diffuse = diffuse_color * Fd_Burley(input, &derived_input);
    }

// #ifdef STANDARD_MATERIAL_ANISOTROPY
    // let specular_light = specular_anisotropy(input, &derived_input, L, roughness, 1.0);
// #else   // STANDARD_MATERIAL_ANISOTROPY
    // let specular_light = specular(input, &derived_input, roughness, 1.0);
// #endif  // STANDARD_MATERIAL_ANISOTROPY

#ifdef STANDARD_MATERIAL_CLEARCOAT
    let clearcoat_N = (*input).layers[LAYER_CLEARCOAT].N;
    let clearcoat_strength = (*input).clearcoat_strength;
    let clearcoat_roughness = (*input).layers[LAYER_CLEARCOAT].roughness;

    // Perform specular input calculations again for the clearcoat layer. We
    // can't reuse the above because the clearcoat normal might be different
    // from the main layer normal.
    var derived_clearcoat_input = derive_lighting_input(clearcoat_N, V, L);

    let Fc_Frc =
        specular_clearcoat(input, &derived_clearcoat_input, clearcoat_strength, clearcoat_roughness, 1.0);
    let inv_Fc = 1.0 - Fc_Frc.r;
    let Frc = Fc_Frc.g;
#endif  // STANDARD_MATERIAL_CLEARCOAT

    var color: vec3<f32>;
// #ifdef STANDARD_MATERIAL_CLEARCOAT
    // Account for the Fresnel term from the clearcoat darkening the main layer.
    //
    // <https://google.github.io/filament/Filament.html#materialsystem/clearcoatmodel/integrationinthesurfaceresponse>
    // color = (diffuse + specular_light * inv_Fc) * inv_Fc * derived_input.NdotL +
        // Frc * derived_clearcoat_input.NdotL;
// #else   // STANDARD_MATERIAL_CLEARCOAT
    // color = (diffuse + specular_light) * derived_input.NdotL;
    color = (diffuse) * derived_input.NdotL;
// #endif  // STANDARD_MATERIAL_CLEARCOAT

    var texture_sample = 1f;

color *= (*light).color.rgb * texture_sample;

#ifdef ATMOSPHERE
    let P = (*input).P;
    // let atmosphere = view_bindings::atmosphere_data.atmosphere;
    // let O = vec3(0.0, atmosphere.bottom_radius, 0.0);
    // let P_scaled = P * vec3(view_bindings::atmosphere_data.settings.scene_units_to_m);
    // let P_as = P_scaled + O;
    // let P_clamped = clamp_to_surface(atmosphere, P_as);
    // let r = length(P_clamped);
    // let local_up = normalize(P_clamped);
    // let mu_light = dot(L, local_up);

    // // Sample atmosphere
    // let transmittance = sample_transmittance_lut(r, mu_light);
    // let sun_visibility = calculate_visible_sun_ratio(atmosphere, r, mu_light, (*light).sun_disk_angular_size);
    
    // // Apply atmospheric effects
    // color *= transmittance * sun_visibility;
#endif

    return color;
}

// Input to a lighting function for a single layer (either the base layer or the
// clearcoat layer).
struct LayerLightingInput {
    // The normal vector.
    N: vec3<f32>,
    // The reflected vector.
    R: vec3<f32>,
    // The normal vector ⋅ the view vector.
    NdotV: f32,

    // The perceptual roughness of the layer.
    perceptual_roughness: f32,
    // The roughness of the layer.
    roughness: f32,
}

// Input to a lighting function (`point_light`, `spot_light`,
// `directional_light`).
struct LightingInput {
#ifdef STANDARD_MATERIAL_CLEARCOAT
    layers: array<LayerLightingInput, 2>,
#else   // STANDARD_MATERIAL_CLEARCOAT
    layers: array<LayerLightingInput, 1>,
#endif  // STANDARD_MATERIAL_CLEARCOAT

    // The world-space position.
    P: vec3<f32>,
    // The vector to the view.
    V: vec3<f32>,

    // // The diffuse color of the material.
    // diffuse_color: vec3<f32>,

    // // The 0-1 metallic factor of the material.
    // metallic: f32,

    // // Specular reflectance at the normal incidence angle.
    // F0_dielectric: vec3<f32>,
    // F0_metallic: vec3<f32>,
    
    // Constants for the BRDF approximation.
    //
    // See `EnvBRDFApprox` in
    // <https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile>.
    // What we call `F_ab` they call `AB`.
    F_ab: vec2<f32>,

#ifdef STANDARD_MATERIAL_CLEARCOAT
    // The strength of the clearcoat layer.
    clearcoat_strength: f32,
#endif  // STANDARD_MATERIAL_CLEARCOAT

#ifdef STANDARD_MATERIAL_ANISOTROPY
    // The anisotropy strength, reflecting the amount of increased roughness in
    // the tangent direction.
    anisotropy: f32,
    // The tangent direction for anisotropy: i.e. the direction in which
    // roughness increases.
    Ta: vec3<f32>,
    // The bitangent direction, which is the cross product of the normal with
    // the tangent direction.
    Ba: vec3<f32>,
#endif  // STANDARD_MATERIAL_ANISOTROPY
}

// Values derived from the `LightingInput` for both diffuse and specular lights.
struct DerivedLightingInput {
    // The half-vector between L, the incident light vector, and V, the view
    // vector.
    H: vec3<f32>,
    // The normal vector ⋅ the incident light vector.
    NdotL: f32,
    // The normal vector ⋅ the half-vector.
    NdotH: f32,
    // The incident light vector ⋅ the half-vector.
    LdotH: f32,
}

fn derive_lighting_input(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>) -> DerivedLightingInput {
    var input: DerivedLightingInput;
    var H: vec3<f32> = normalize(L + V);
    input.H = H;
    input.NdotL = saturate(dot(N, L));
    input.NdotH = saturate(dot(N, H));
    input.LdotH = saturate(dot(L, H));
    return input;
}

const PI: f32 = 3.1415926535897932384626433832795;

// Diffuse BRDF
// https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
// fd(v,l) = σ/π * 1 / { |n⋅v||n⋅l| } ∫Ω D(m,α) G(v,l,m) (v⋅m) (l⋅m) dm
//
// simplest approximation
// float Fd_Lambert() {
//     return 1.0 / PI;
// }
//
// vec3 Fd = diffuseColor * Fd_Lambert();
//
// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
fn Fd_Burley(
    input: ptr<function, LightingInput>,
    derived_input: ptr<function, DerivedLightingInput>,
) -> f32 {
    // Unpack.
    let roughness = (*input).layers[LAYER_BASE].roughness;
    let NdotV = (*input).layers[LAYER_BASE].NdotV;
    let NdotL = (*derived_input).NdotL;
    let LdotH = (*derived_input).LdotH;

    let f90 = 0.5 + 2.0 * roughness * LdotH * LdotH;
    let lightScatter = F_Schlick(1.0, f90, NdotL);
    let viewScatter = F_Schlick(1.0, f90, NdotV);
    return lightScatter * viewScatter * (1.0 / PI);
}

fn F_Schlick(f0: f32, f90: f32, VdotH: f32) -> f32 {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow(1.0 - VdotH, 5.0);
}