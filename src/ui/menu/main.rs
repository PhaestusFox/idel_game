pub use super::*;

pub struct MainMenu;

impl Menu for MainMenu {
    const MENU_ID: MenuId = MenuId::new("main_menu");
    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_main_menu, clear_stack).into_configs()
    }
}

fn open_main_menu(mut builder: super::MenuBuilder) {
    builder.label("Main Menu");
    builder.button_with_props(
        "Play",
        to_play,
        ButtonProps {
            variant: ButtonVariant::Primary,
            corners: feathers::rounded_corners::RoundedCorners::Top,
        },
    );
    builder.button("Settings", open_menu::<settings_menu::SettingsMenu>());
    builder.button("Quit", quit);
}
