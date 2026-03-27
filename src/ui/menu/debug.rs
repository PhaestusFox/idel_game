use bevy::{diagnostic::DiagnosticsStore, window::PrimaryWindow};

use crate::{
    map::ChunkId,
    player::{MoveMode, PlayerEntity},
};

use super::*;

pub struct DebugMenu;

impl Menu for DebugMenu {
    const MENU_ID: MenuId = MenuId::new("debug_menu");

    fn init(app: &mut App) {
        if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        }
        app.add_plugins(biome_seeker::BiomeDebugPlugin);
        app.add_systems(Update, update_fps);
        app.add_systems(Update, update_pos_text);
        app.add_systems(Startup, spawn_console);
    }

    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (open_debug_menu).into_configs()
    }
}
#[derive(Component)]
pub struct DebugConsole;

fn spawn_console(mut commands: Commands) {
    commands.spawn((
        Name::new("Debug Console"),
        DebugConsole,
        Node {
            top: Val::Px(10.),
            left: Val::Px(10.),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
    ));
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
        clean.clone(),
    ));

    if *mode.get() == MoveMode::Fly {
        fly.insert(Checked);
    }

    root.with_child((
        checkbox((), Spawn((Text::new("Open Map Debug Console"), ThemedText))),
        observe(crate::map::debug::toggle_console),
        clean.clone(),
    ));
    root.with_child((
        Node::DEFAULT,
        children![
            (
                checkbox((), Spawn((Text::new("Show Local X,Y,Z"), ThemedText))),
                observe(toggle_local),
                PosText::Local,
                clean.clone(),
            ),
            (
                checkbox((), Spawn((Text::new("Show Global X,Y,Z"), ThemedText))),
                observe(toggle_local),
                PosText::Global,
                clean.clone(),
            ),
            (
                checkbox((), Spawn((Text::new("Show Offset"), ThemedText))),
                observe(toggle_local),
                PosText::Offset,
                clean.clone(),
            ),
            (
                checkbox((), Spawn((Text::new("Show ChunkId"), ThemedText))),
                observe(toggle_local),
                PosText::ChunkId,
                clean.clone(),
            ),
        ],
    ));

    root.with_child((
        Node::DEFAULT,
        children![(
            checkbox((), Spawn((Text::new("Enable Biome Seeker"), ThemedText))),
            observe(biome_seeker::toggle_biome_seeker),
            clean.clone(),
        ),],
    ));
}

#[derive(Component)]
struct FPSText;

fn toggle_fps(
    trigger: On<ValueChange<bool>>,
    mut commands: Commands,
    fps: Query<Entity, With<FPSText>>,
    console: Single<Entity, With<DebugConsole>>,
) {
    if trigger.value {
        commands.entity(trigger.source).insert(Checked);
        spawn_fps(&mut commands, *console);
    } else {
        commands.entity(trigger.source).remove::<Checked>();
        for fps in &fps {
            commands.entity(fps).despawn();
        }
    }
}

fn spawn_fps(commands: &mut Commands, console: Entity) {
    commands.spawn((
        FPSText,
        Node {
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        children![
            (Text::new("FPS: N/A"), ThemedText,),
            (Text::new("FPS_A: N/A"), ThemedText,),
            // (Text::new("FPS_S: N/A"), ThemedText,),
        ],
        ChildOf(console),
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
    // let Ok(mut fps) = text.get_mut(fps_root[0]) else {
    //     error!("FPS text entity was despawned");
    //     return;
    // };
    // fps.0 = format!("FPS: {:.2}", diagnostic.value().unwrap_or_default());
    let Ok(mut fps_s) = text.get_mut(fps_root[0]) else {
        error!("FPS_S text entity was despawned");
        return;
    };
    fps_s.0 = format!("FPS_S: {:.2}", diagnostic.smoothed().unwrap_or_default());
    let Ok(mut fps_a) = text.get_mut(fps_root[1]) else {
        error!("FPS_A text entity was despawned");
        return;
    };
    fps_a.0 = format!("(A:{:.2})", diagnostic.average().unwrap_or_default());
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

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum PosText {
    Local,
    Global,
    Offset,
    ChunkId,
}

fn toggle_local(
    trigger: On<ValueChange<bool>>,
    mut commands: Commands,
    text: Query<(Entity, &PosText)>,
    console: Single<Entity, With<DebugConsole>>,
) {
    let (_, pos) = text.get(trigger.source).unwrap();
    if trigger.value {
        commands.entity(trigger.source).insert(Checked);
        commands.spawn((
            *pos,
            Text::new(format!("{:?}: N/A", pos)),
            ThemedText,
            ChildOf(*console),
        ));
    } else {
        commands.entity(trigger.source).remove::<Checked>();
        for (entity, text_pos) in &text {
            if *pos == *text_pos && entity != trigger.source {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn update_pos_text(
    mut pos_text: Query<(&PosText, &mut Text)>,
    player: Single<&Transform, With<PlayerEntity>>,
    offset: Res<ChunkId>,
) {
    for (pos, mut text) in &mut pos_text {
        match pos {
            PosText::Local => {
                text.0 = format!(
                    "Local: X:{:.2}, Y:{:.2}, Z:{:.2}",
                    player.translation.x, player.translation.y, player.translation.z
                );
            }
            PosText::Global => {
                let g = player.translation + offset.offset();
                text.0 = format!("Global: X:{:.2}, Y:{:.2}, Z:{:.2}", g.x, g.y, g.z);
            }
            PosText::Offset => {
                text.0 = format!("Offset by {}, {}, {}", offset.x, offset.y, offset.z);
            }
            PosText::ChunkId => {
                text.0 = format!(
                    "In {}",
                    ChunkId::from_translation(player.translation) + *offset
                );
            }
        }
    }
}

mod biome_seeker;
