use crate::player::{CameraSettings, FlyCameraSettings};

use super::*;

pub struct CameraSettingsMenu;

impl Menu for CameraSettingsMenu {
    const MENU_ID: MenuId = MenuId::new("camera_settings_menu");

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_camera_settings_menu).into_configs()
    }
}

fn open_camera_settings_menu(
    settings: Res<CameraSettings>,
    fly_settings: Res<FlyCameraSettings>,
    mut menu: super::MenuBuilder,
) {
    menu.label("Camera Settings");
    menu.vertical()
        .label("Sensitivity")
        .add_slider(
            |change: On<ValueChange<f32>>,
             mut settings: ResMut<CameraSettings>,
             mut commands: Commands| {
                settings.look_sensitivity = change.value;
                commands
                    .entity(change.source)
                    .insert(SliderValue(settings.look_sensitivity));
            },
            SliderSettings::new(settings.look_sensitivity, 0.1, 0.01)
                .with_precision(2)
                .with_step(0.01),
        )
        .add_checkbox_with_state(
            "Invert Y",
            |check: On<ValueChange<bool>>,
             mut settings: ResMut<CameraSettings>,
             mut commands: Commands| {
                settings.invert_look_y = check.value;
                if check.value {
                    commands.entity(check.source).insert(Checked);
                } else {
                    commands.entity(check.source).remove::<Checked>();
                }
            },
            settings.invert_look_y,
        );
    menu.vertical().label("Move Speed").add_slider(
        |change: On<ValueChange<f32>>,
         mut settings: ResMut<FlyCameraSettings>,
         mut commands: Commands| {
            settings.move_speed = change.value;
            commands
                .entity(change.source)
                .insert(SliderValue(settings.move_speed));
        },
        SliderSettings::new(5., 20., fly_settings.move_speed).with_precision(1),
    );

    menu.vertical().label("Boost X").add_slider(
        |change: On<ValueChange<f32>>,
         mut settings: ResMut<FlyCameraSettings>,
         mut commands: Commands| {
            settings.boost_multiplier = change.value;
            commands
                .entity(change.source)
                .insert(SliderValue(settings.boost_multiplier));
        },
        SliderSettings::new(0.5, 5.0, fly_settings.boost_multiplier)
            .with_precision(1)
            .with_step(0.5),
    );
}
