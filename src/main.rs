use bevy::{
    color::palettes::css::RED,
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};
use player::PlayerPlugin;
use ui::UiPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    address_mode_w: bevy::image::ImageAddressMode::Repeat,
                    ..default()
                },
            })
            .set(AssetPlugin {
                // file_path: "../../assets".to_string(),
                ..default()
            }),
    );
    app.add_plugins((PlayerPlugin, UiPlugin, map::MapPlugin));
    app.add_plugins(rendering::VoxelRenderingPlugin);
    app.init_state::<GameState>();
    app.run();
}

mod map;
mod player;
mod rendering;
mod ui;

#[derive(States, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    InMenu,
    Playing,
}
