use super::*;

mod camera_settings;
mod keybindings;
pub struct SettingsMenu;

impl Menu for SettingsMenu {
    const MENU_ID: MenuId = MenuId::new("settings_menu");

    fn init(app: &mut App) {
        app.add_menu::<camera_settings::CameraSettingsMenu>();
    }

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_settings_menu).into_configs()
    }
}

fn open_settings_menu(mut builder: super::MenuBuilder) {
    builder.label("Settings");
    builder.button("Camera", open_menu::<camera_settings::CameraSettingsMenu>());
}
