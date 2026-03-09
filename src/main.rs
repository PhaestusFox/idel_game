use bevy::prelude::*;
use player::PlayerPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PlayerPlugin);
    app.run();
}

mod map;
mod player;

fn spawn_test_cube(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("TestCube"),
        Mesh3d(asset_server.add(Mesh::from(Cuboid::from_length(1.)))),
    ));
}
