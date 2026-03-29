use super::*;
pub struct EscapeMenu;

impl Menu for EscapeMenu {
    const MENU_ID: MenuId = MenuId::new_auto_close("esc_menu");

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_escape).into_configs()
    }
}

fn open_escape(mut builder: super::MenuBuilder) {
    builder.label("Escape Menu");
    builder.button_with_props(
        "Resume",
        to_play,
        ButtonProps {
            variant: ButtonVariant::Primary,
            corners: feathers::rounded_corners::RoundedCorners::Top,
        },
    );
    builder.button("Settings", open_menu::<settings_menu::SettingsMenu>());
    builder.button("Debug Menu", open_menu::<debug::DebugMenu>());
    builder.button("Main menu", open_menu::<main::MainMenu>());
}
