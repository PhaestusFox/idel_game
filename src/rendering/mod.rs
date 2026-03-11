mod voxel_shader;

use bevy::prelude::*;

pub use voxel_shader::*;
pub struct VoxelRenderingPlugin;

impl Plugin for VoxelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<CustomMaterial>::default());
    }
}
