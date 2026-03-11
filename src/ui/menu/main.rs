pub use super::*;

#[derive(Resource)]
pub struct MainMenu {
    on_play: SystemId,
    open_settings: SystemId,
}

impl Menu for MainMenu {
    const MENU_ID: MenuId = MenuId::new("main_menu");
    fn init(app: &mut App) {
        app.init_resource::<Self>();
    }
    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_main_menu, clear_stack).into_configs()
    }
}

impl FromWorld for MainMenu {
    fn from_world(world: &mut World) -> Self {
        let on_play = world.register_system(|mut state: ResMut<NextState<GameState>>| {
            state.set(GameState::Playing);
        });
        let open_settings = world.register_system(open_menu::<settings_menu::SettingsMenu>());
        Self {
            on_play,
            open_settings,
        }
    }
}

fn open_main_menu(
    mut commands: Commands,
    menu: Res<MainMenu>,
    root: Single<Entity, With<MenuRoot>>,
) {
    let quit = MenuAction::from_commands(&mut commands, |mut quit: MessageWriter<AppExit>| {
        quit.write(AppExit::Success);
    });
    let dso = DespawnOnExit(MainMenu::id());
    commands.entity(*root).insert(children![
        button(
            ButtonProps {
                variant: ButtonVariant::Primary,
                corners: feathers::rounded_corners::RoundedCorners::Top
            },
            (dso.clone(), MenuAction::new(menu.on_play)),
            Spawn((Text::new("Play"), ThemedText))
        ),
        button(
            ButtonProps {
                variant: ButtonVariant::Normal,
                corners: feathers::rounded_corners::RoundedCorners::None
            },
            (dso.clone(), MenuAction::new(menu.open_settings)),
            Spawn((Text::new("Settings"), ThemedText))
        ),
        button(
            ButtonProps {
                variant: ButtonVariant::Normal,
                corners: feathers::rounded_corners::RoundedCorners::Bottom
            },
            (dso, quit),
            Spawn((Text::new("Quit"), ThemedText))
        )
    ]);
}
