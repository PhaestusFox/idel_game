mod voxel_shader;

use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, ViewNode, ViewNodeRunner,
        },
        renderer::RenderContext,
    },
};

pub use voxel_shader::*;
pub struct VoxelRenderingPlugin;

impl Plugin for VoxelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<CustomMaterial>::default());

        let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };
    }
}
