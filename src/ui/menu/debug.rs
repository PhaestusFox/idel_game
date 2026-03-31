use bevy::{
    diagnostic::DiagnosticsStore,
    window::{PresentMode, PrimaryWindow},
};

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

struct DebugMenuState {
    show_fps: bool,
    cap_60: bool,
    fly_mode: bool,
    console_open: bool,
    show_local: bool,
    show_global: bool,
    show_offset: bool,
    show_chunk_id: bool,
    show_biome_seeker: bool,
}

unsafe impl SystemParam for DebugMenuState {
    type Item<'world, 'state> = DebugMenuState;
    type State = ();
    fn init_access(
        _state: &Self::State,
        _system_meta: &mut bevy::ecs::system::SystemMeta,
        component_access_set: &mut bevy::ecs::query::FilteredAccessSet,
        _world: &mut World,
    ) {
        component_access_set.read_all();
    }
    fn init_state(_world: &mut World) -> Self::State {}
    unsafe fn get_param<'world, 'state>(
        _state: &'state mut Self::State,
        _system_meta: &bevy::ecs::system::SystemMeta,
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        _change_tick: bevy::ecs::change_detection::Tick,
    ) -> Self::Item<'world, 'state> {
        let mut state = DebugMenuState {
            show_fps: false,
            cap_60: false,
            fly_mode: false,
            console_open: false,
            show_local: false,
            show_global: false,
            show_offset: false,
            show_chunk_id: false,
            show_biome_seeker: false,
        };
        unsafe {
            state.show_fps = world
                .world_mut()
                .query_filtered::<(), With<FPSText>>()
                .query(world.world())
                .count()
                > 0;
            state.fly_mode = world
                .get_resource()
                .is_some_and(|mode: &State<MoveMode>| *mode == MoveMode::Fly);
            for pos in world
                .world_mut()
                .query_filtered::<&PosText, Without<bevy::ui_widgets::Checkbox>>()
                .query(world.world())
            {
                match pos {
                    PosText::Local => state.show_local = true,
                    PosText::Global => state.show_global = true,
                    PosText::Offset => state.show_offset = true,
                    PosText::ChunkId => state.show_chunk_id = true,
                }
            }
            state.show_biome_seeker = world
                .world_mut()
                .query_filtered::<(), With<biome_seeker::BiomeInfoText>>()
                .query(world.world())
                .count()
                > 0;
            state.cap_60 = world
                .world_mut()
                .query_filtered::<&Window, With<PrimaryWindow>>()
                .query(world.world())
                .iter()
                .any(|w| w.present_mode == PresentMode::AutoVsync);
            state.console_open = world
                .world_mut()
                .query_filtered::<&Children, With<crate::map::debug::MapDebugConsole>>()
                .query(world.world())
                .iter()
                .any(|c| !c.is_empty());
        }
        state
    }
}

fn open_debug_menu(mut menu: super::MenuBuilder, state: DebugMenuState) {
    menu.add_checkbox_with_state("Show FPS", toggle_fps, state.show_fps);
    menu.add_checkbox_with_state("Cap 60 FPS", toggle_cap, state.cap_60);
    menu.add_checkbox_with_state("Fly Mode", toggle_fly, state.fly_mode);
    menu.add_checkbox_with_state(
        "Open Map Debug Console",
        crate::map::debug::toggle_console,
        state.console_open,
    );
    let mut pos = menu.vertical();
    pos.label("Show Cowoadants");
    let mut pos = pos.horizontal();
    if state.show_local {
        pos.add_checkbox_with_ext("Local X,Y,Z", toggle_local, (PosText::Local, Checked));
    } else {
        pos.add_checkbox_with_ext("Local X,Y,Z", toggle_local, PosText::Local);
    }
    if state.show_global {
        pos.add_checkbox_with_ext("Global X,Y,Z", toggle_local, (PosText::Global, Checked));
    } else {
        pos.add_checkbox_with_ext("Global X,Y,Z", toggle_local, PosText::Global);
    }
    if state.show_offset {
        pos.add_checkbox_with_ext("Offset", toggle_local, (PosText::Offset, Checked));
    } else {
        pos.add_checkbox_with_ext("Offset", toggle_local, PosText::Offset);
    }
    if state.show_chunk_id {
        pos.add_checkbox_with_ext("ChunkId", toggle_local, (PosText::ChunkId, Checked));
    } else {
        pos.add_checkbox_with_ext("ChunkId", toggle_local, PosText::ChunkId);
    }

    let mut pos = menu.horizontal();
    pos.add_checkbox_with_state(
        "Show Biome Seeker",
        biome_seeker::toggle_biome_seeker,
        state.show_biome_seeker,
    );
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
