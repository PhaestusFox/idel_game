use bevy::{color::palettes::css::RED, prelude::*};
use player::PlayerPlugin;
use ui::UiPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins((PlayerPlugin, UiPlugin, map::MapPlugin));
    app.add_systems(Startup, spawn_test_cube);
    app.init_state::<GameState>();
    app.run();
}

mod map;
mod player;
mod ui;

fn spawn_test_cube(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("TestCube"),
        Mesh3d(asset_server.add(Mesh::from(Cuboid::from_length(1.)))),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: RED.into(),
            ..default()
        })),
    ));
}

#[derive(States, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    InMenu,
    Playing,
}
