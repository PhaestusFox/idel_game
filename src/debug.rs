use crate::rendering::CustomMaterial;

use super::*;

/// All hail the debug cube
pub fn spawn_test_cube(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chunk = crate::map::ChunkData::test();
    let image = chunk.to_image();
    commands.spawn((
        Name::new("TestCube"),
        Mesh3d(asset_server.add(Cuboid::from_length(32.).into())),
        MeshMaterial3d(asset_server.add(CustomMaterial {
            lod: 1.,
            color_texture: Some(asset_server.load("colors.png")),
            alpha_mode: AlphaMode::Opaque,
            chunk_offset: Vec3::ZERO,
            data: asset_server.add(image),
        })),
        Transform::from_xyz(0., 0., 0.),
    ));
}
