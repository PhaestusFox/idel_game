use super::*;
pub struct EscapeMenu;

impl Menu for EscapeMenu {
    const MENU_ID: MenuId = MenuId::new_auto_close("esc_menu");

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_escape).into_configs()
    }
}

fn open_escape(mut commands: Commands, root: Single<Entity, With<MenuRoot>>) {
    let quit = MenuAction::from_commands(&mut commands, open_menu::<main::MainMenu>());
    let resume = MenuAction::from_commands(&mut commands, to_play);
    let open_settings =
        MenuAction::from_commands(&mut commands, open_menu::<settings_menu::SettingsMenu>());
    let dso = DespawnOnExit(EscapeMenu::id());

    commands.entity(*root).insert(children![
        button(
            ButtonProps {
                variant: ButtonVariant::Primary,
                corners: feathers::rounded_corners::RoundedCorners::Top,
            },
            (dso.clone(), resume),
            Spawn((Text::new("Resume"), ThemedText))
        ),
        button(
            ButtonProps {
                variant: ButtonVariant::Normal,
                corners: feathers::rounded_corners::RoundedCorners::None,
            },
            (dso.clone(), open_settings),
            Spawn((Text::new("Settings"), ThemedText))
        ),
        button(
            ButtonProps {
                variant: ButtonVariant::Normal,
                corners: feathers::rounded_corners::RoundedCorners::Bottom,
            },
            (dso, quit),
            Spawn((Text::new("Main Menu"), ThemedText))
        ),
    ]);
}
