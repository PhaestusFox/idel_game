//! A shader and a material that uses it.

use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/vox.wgsl";
const SHADER_VERTEX_PATH: &str = "shaders/vertex.wgsl";

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[texture(1, dimension = "2d")]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    #[uniform(3)]
    pub lod: u32,
    #[storage_texture(4, dimension = "3d", image_format = Rgba8Uint, access = ReadOnly, visibility(fragment, vertex))]
    pub data: Handle<Image>,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn vertex_shader() -> ShaderRef {
        SHADER_VERTEX_PATH.into()
    }

    fn enable_prepass() -> bool {
        true
    }

    fn enable_shadows() -> bool {
        false
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn deferred_vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}
