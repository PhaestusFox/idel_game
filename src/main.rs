use bevy::{
    color::palettes::css::RED,
    image::{ImageSampler, ImageSamplerDescriptor},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use player::PlayerPlugin;
use ui::UiPlugin;

fn main() {
    let mut app = App::new();
    let mut tpo = TaskPoolOptions::default();
    tpo.async_compute.max_threads = usize::MAX;
    tpo.async_compute.percent = 0.75;
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    address_mode_w: bevy::image::ImageAddressMode::Repeat,
                    mag_filter: bevy::image::ImageFilterMode::Linear,
                    min_filter: bevy::image::ImageFilterMode::Nearest,
                    mipmap_filter: bevy::image::ImageFilterMode::Linear,
                    ..default()
                },
            })
            .set(TaskPoolPlugin {
                task_pool_options: tpo,
            }),
    );
    app.add_systems(
        Update,
        debug::spawn_test_cube.run_if(input_just_pressed(KeyCode::F10)),
    );
    app.add_plugins((PlayerPlugin, UiPlugin, map::MapPlugin));
    app.add_plugins(rendering::VoxelRenderingPlugin);
    app.init_state::<GameState>();
    app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    app.run();
}

mod debug;
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
