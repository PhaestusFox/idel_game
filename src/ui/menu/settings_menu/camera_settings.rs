use bevy::{
    ui::Checked,
    ui_widgets::{SliderPrecision, SliderStep, SliderValue, ValueChange, observe},
};

use crate::player::FlyCameraSettings;

use super::*;

pub struct CameraSettingsMenu;

impl Menu for CameraSettingsMenu {
    const MENU_ID: MenuId = MenuId::new("camera_settings_menu");

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_camera_settings_menu).into_configs()
    }
}

fn open_camera_settings_menu(
    mut commands: Commands,
    root: Single<Entity, With<MenuRoot>>,
    settings: Res<FlyCameraSettings>,
) {
    let dso = DespawnOnExit(CameraSettingsMenu::id());
    let mut root = commands.entity(*root);
    root.with_child((
        slider(
            SliderProps {
                value: settings.look_sensitivity,
                min: 0.01,
                max: 0.1,
            },
            (SliderStep(0.01), SliderPrecision(2)),
        ),
        observe(
            |change: On<ValueChange<f32>>,
             mut settings: ResMut<FlyCameraSettings>,
             mut commands: Commands| {
                settings.look_sensitivity = change.value;
                commands
                    .entity(change.source)
                    .insert(SliderValue(settings.look_sensitivity));
            },
        ),
        dso.clone(),
    ));

    root.with_child((
        slider(
            SliderProps {
                value: settings.move_speed,
                min: 5.,
                max: 20.,
            },
            (SliderStep(0.5), SliderPrecision(1)),
        ),
        observe(
            |change: On<ValueChange<f32>>,
             mut settings: ResMut<FlyCameraSettings>,
             mut commands: Commands| {
                settings.move_speed = change.value;
                commands
                    .entity(change.source)
                    .insert(SliderValue(settings.move_speed));
            },
        ),
        dso.clone(),
    ));

    root.with_child((
        slider(
            SliderProps {
                value: settings.boost_multiplier,
                min: 0.5,
                max: 5.0,
            },
            (SliderStep(0.5), SliderPrecision(1)),
        ),
        observe(
            |change: On<ValueChange<f32>>,
             mut settings: ResMut<FlyCameraSettings>,
             mut commands: Commands| {
                settings.boost_multiplier = change.value;
                commands
                    .entity(change.source)
                    .insert(SliderValue(settings.boost_multiplier));
            },
        ),
        dso.clone(),
    ));

    let invert_look_y = root.with_child((
        dso,
        observe(
            |check: On<ValueChange<bool>>,
             mut settings: ResMut<FlyCameraSettings>,
             mut commands: Commands| {
                settings.invert_look_y = check.value;
                if check.value {
                    commands.entity(check.source).insert(Checked);
                } else {
                    commands.entity(check.source).remove::<Checked>();
                }
            },
        ),
        checkbox((), Spawn((Text::new("Invert Look Y"), ThemedText))),
    ));
    if settings.invert_look_y {
        invert_look_y.insert(Checked);
    }
}
