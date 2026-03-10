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

fn open_settings_menu(mut commands: Commands, root: Single<Entity, With<MenuRoot>>) {
    let camera_setting = MenuAction::from_commands(
        &mut commands,
        open_menu::<camera_settings::CameraSettingsMenu>(),
    );
    let dso = DespawnOnExit(SettingsMenu::id());
    commands.entity(*root).insert(children![button(
        ButtonProps {
            variant: ButtonVariant::Normal,
            corners: feathers::rounded_corners::RoundedCorners::All
        },
        (dso, camera_setting),
        Spawn((Text::new("Camera Settings"), ThemedText))
    ),]);
}
