use bevy::{diagnostic::DiagnosticsStore, window::PrimaryWindow};

use super::*;

pub struct DebugMenu;

impl Menu for DebugMenu {
    const MENU_ID: MenuId = MenuId::new("debug_menu");

    fn init(app: &mut App) {
        if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        }
        app.add_systems(Update, update_fps);
    }

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_debug_menu).into_configs()
    }
}

fn open_debug_menu(mut commands: Commands, root: Single<Entity, With<MenuRoot>>) {
    let mut root = commands.entity(*root);
    let clean = DespawnOnExit(DebugMenu::id());
    root.with_child((
        checkbox((), Spawn((Text::new("Show FPS"), ThemedText))),
        observe(toggle_fps),
        clean.clone(),
    ));
    root.with_child((
        checkbox((), Spawn((Text::new("Cap 60 FPS"), ThemedText))),
        observe(toggle_cap),
        clean,
    ));
}

#[derive(Component)]
struct FPSText;

fn toggle_fps(
    trigger: On<ValueChange<bool>>,
    mut commands: Commands,
    fps: Query<Entity, With<FPSText>>,
) {
    if trigger.value {
        commands.entity(trigger.source).insert(Checked);
        spawn_fps(&mut commands);
    } else {
        commands.entity(trigger.source).remove::<Checked>();
        for fps in &fps {
            commands.entity(fps).despawn();
        }
    }
}

fn spawn_fps(commands: &mut Commands) {
    commands.spawn((
        FPSText,
        Node {
            top: Val::Px(10.),
            left: Val::Px(10.),
            ..Default::default()
        },
        Text::new("FPS: N/A"),
        ThemedText,
    ));
}

fn update_fps(mut fps: Single<&mut Text, With<FPSText>>, diagnostics: Res<DiagnosticsStore>) {
    fps.0 = format!(
        "FPS: {:.1}",
        diagnostics
            .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.value())
            .unwrap_or(0.0)
    );
}

fn toggle_cap(
    trigger: On<ValueChange<bool>>,
    mut commands: Commands,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if trigger.value {
        commands.entity(trigger.source).insert(Checked);
        // enable cap
    } else {
        commands.entity(trigger.source).remove::<Checked>();
        // disable cap
        window.present_mode = bevy::window::PresentMode::AutoNoVsync;
    }
}
