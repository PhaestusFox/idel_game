use bevy::{
    prelude::*,
    ui_widgets::{SliderValue, ValueChange},
};

use crate::{
    map::{GenerationDistance, PersistenceDistance},
    player::RenderDistance,
    ui::*,
};

pub struct ViewMenu;

impl Menu for ViewMenu {
    const MENU_ID: MenuId = MenuId::new("view_menu");
    fn open() -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::ScheduleSystem> {
        open_view_menu.into_configs()
    }
}

fn open_view_menu(
    mut builder: MenuBuilder,
    view_distance: Res<RenderDistance>,
    generation_distance: Res<GenerationDistance>,
) {
    builder.label("View");
    builder.vertical().label("View Distance").add_slider(
        change_view_distance,
        SliderSettings::new(1., 32., view_distance.0 as f32)
            .with_step(1.)
            .with_precision(0),
    );
    builder.vertical().label("Generation Distance").add_slider(
        change_generation_distance,
        SliderSettings::new(1., 32., **generation_distance as f32)
            .with_step(1.)
            .with_precision(0),
    );
}

fn change_view_distance(
    value: On<ValueChange<f32>>,
    mut settings: ResMut<RenderDistance>,
    mut commands: Commands,
) {
    commands
        .entity(value.source)
        .insert(SliderValue(value.value));
    settings.0 = value.value as u32;
}

fn change_generation_distance(
    value: On<ValueChange<f32>>,
    mut settings: ResMut<GenerationDistance>,
    mut persistance: ResMut<PersistenceDistance>,
    mut commands: Commands,
) {
    commands
        .entity(value.source)
        .insert(SliderValue(value.value));
    **settings = value.value as u32;
    **persistance = value.value as u32 + 2;
}
