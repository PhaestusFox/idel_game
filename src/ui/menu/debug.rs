use bevy::{
    diagnostic::DiagnosticsStore, log::tracing_subscriber::fmt::format, window::PrimaryWindow,
};

use crate::player::MoveMode;

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

fn open_debug_menu(
    mut commands: Commands,
    root: Single<Entity, With<MenuRoot>>,
    mode: Res<State<MoveMode>>,
) {
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
        clean.clone(),
    ));
    let fly = root.with_child((
        checkbox((), Spawn((Text::new("Fly"), ThemedText))),
        observe(toggle_fly),
        clean,
    ));
    if *mode.get() == MoveMode::Fly {
        fly.insert(Checked);
    }
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
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
        children![
            (Text::new("FPS: N/A"), ThemedText,),
            (Text::new("FPS_A: N/A"), ThemedText,),
            (Text::new("FPS_S: N/A"), ThemedText,),
        ],
    ));
}

fn update_fps(
    fps_root: Single<&Children, With<FPSText>>,
    mut text: Query<&mut Text>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let diagnostic = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .unwrap();
    let Ok(mut fps) = text.get_mut(fps_root[0]) else {
        error!("FPS text entity was despawned");
        return;
    };
    fps.0 = format!("FPS: {:.2}", diagnostic.value().unwrap_or_default());
    let Ok(mut fps_a) = text.get_mut(fps_root[1]) else {
        error!("FPS_A text entity was despawned");
        return;
    };
    fps_a.0 = format!("FPS_A: {:.2}", diagnostic.average().unwrap_or_default());
    let Ok(mut fps_s) = text.get_mut(fps_root[2]) else {
        error!("FPS_S text entity was despawned");
        return;
    };
    fps_s.0 = format!("FPS_S: {:.2}", diagnostic.smoothed().unwrap_or_default());
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

fn toggle_fly(
    trigger: On<ValueChange<bool>>,
    mut commands: Commands,
    mut mode: ResMut<NextState<MoveMode>>,
) {
    if trigger.value {
        commands.entity(trigger.source).insert(Checked);
        mode.set(MoveMode::Fly);
    } else {
        commands.entity(trigger.source).remove::<Checked>();
        mode.set(MoveMode::Walk);
    }
}
