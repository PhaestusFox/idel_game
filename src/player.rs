use bevy::{
    input::{common_conditions::input_just_pressed, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};

use crate::{
    GameState,
    map::{CHUNK_SIZE, Chunk, ChunkData, ChunkId, ChunkLookup, LoD, MeshGenerator},
    rendering::CustomMaterial,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveBindings>()
            .init_resource::<FlyCameraSettings>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                ((player_look, move_player)
                    .chain()
                    .run_if(in_state(GameState::Playing)),),
            )
            .add_systems(OnEnter(GameState::Playing), hide_cursor)
            .add_systems(OnExit(GameState::Playing), show_cursor);

        app.add_observer(move_the_universe_not_the_ship)
            .add_systems(First, detect_chunk_transition);
    }
}

#[derive(Component)]
#[require(ChunkId)]
pub struct PlayerEntity;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Name::new("PlayerCamera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        PlayerEntity,
        DistanceFog {
            color: Color::srgb(0.25, 0.25, 0.25),
            falloff: FogFalloff::Exponential { density: 0.5 },
            ..default()
        },
    ));
}

#[derive(Resource)]
pub struct MoveBindings {
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub fly_up: KeyCode,
    pub fly_down: KeyCode,
    pub boost: KeyCode,
}

impl Default for MoveBindings {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            fly_up: KeyCode::Space,
            fly_down: KeyCode::ControlLeft,
            boost: KeyCode::ShiftLeft,
        }
    }
}

#[derive(Resource)]
pub struct FlyCameraSettings {
    pub move_speed: f32,
    pub look_sensitivity: f32,
    pub invert_look_y: bool,
    pub boost_multiplier: f32,
}

impl Default for FlyCameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 32.,
            look_sensitivity: 0.05,
            invert_look_y: false,
            boost_multiplier: 33.0,
        }
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    move_bindings: Res<MoveBindings>,
    settings: Res<FlyCameraSettings>,
    mut player: Single<&mut Transform, With<PlayerEntity>>,
) {
    let forward = {
        let mut v = *player.forward();
        v.y = 0.0;
        v.normalize_or_zero()
    };
    let right = {
        let mut v = *player.right();
        v.y = 0.0;
        v.normalize_or_zero()
    };

    let mut input = Vec2::ZERO;
    if keyboard_input.pressed(move_bindings.forward) {
        input.y += 1.0;
    }
    if keyboard_input.pressed(move_bindings.backward) {
        input.y -= 1.0;
    }
    if keyboard_input.pressed(move_bindings.right) {
        input.x += 1.0;
    }
    if keyboard_input.pressed(move_bindings.left) {
        input.x -= 1.0;
    }

    let mut move_dir = (right * input.x + forward * input.y).normalize_or_zero();
    if keyboard_input.pressed(move_bindings.fly_up) {
        move_dir.y += 1.0;
    }
    if keyboard_input.pressed(move_bindings.fly_down) {
        move_dir.y -= 1.0;
    }
    if keyboard_input.pressed(move_bindings.boost) {
        move_dir *= settings.boost_multiplier;
    }
    player.translation += move_dir * settings.move_speed * time.delta_secs();
}

fn toggle_cursor(mut windows: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    match windows.grab_mode {
        bevy::window::CursorGrabMode::None => {
            windows.visible = false;
            windows.grab_mode = bevy::window::CursorGrabMode::Locked;
        }
        bevy::window::CursorGrabMode::Confined | bevy::window::CursorGrabMode::Locked => {
            windows.visible = true;
            windows.grab_mode = bevy::window::CursorGrabMode::None;
        }
    }
}

fn player_look(
    mouse_movement: Res<AccumulatedMouseMotion>,
    mut player: Single<&mut Transform, With<PlayerEntity>>,
    settings: Res<FlyCameraSettings>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
) {
    if matches!(cursor.grab_mode, bevy::window::CursorGrabMode::None) {
        return;
    }
    let mut delta = mouse_movement.delta * settings.look_sensitivity;
    if settings.invert_look_y {
        delta.y = -delta.y;
    }
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    yaw -= delta.x.to_radians();
    pitch -= delta.y.to_radians();
    pitch = pitch.clamp(-core::f32::consts::FRAC_PI_2, core::f32::consts::FRAC_PI_2);
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

pub fn show_cursor(mut windows: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    windows.visible = true;
    windows.grab_mode = bevy::window::CursorGrabMode::None;
}

pub fn hide_cursor(mut windows: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    windows.visible = false;
    windows.grab_mode = bevy::window::CursorGrabMode::Locked;
}

pub fn detect_chunk_transition(
    mut player: Single<(&Transform, &mut ChunkId), With<PlayerEntity>>,
    mut commands: Commands,
) {
    let pos = ChunkId::from_translation(player.0.translation);
    if pos != ChunkId::ZERO {
        println!("Player moved to chunk {:?}", pos);
        *player.1 = *player.1 + pos;
        commands.trigger(MoveWorld(*pos));
    }
}

fn move_the_universe_not_the_ship(
    trigger: On<MoveWorld>,
    mut player: Single<&mut Transform, With<PlayerEntity>>,
    chunks: Query<(&ChunkId, &Chunk)>,
    mats: Query<&MeshMaterial3d<CustomMaterial>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    chunk_data: Res<Assets<ChunkData>>,
    lookup: Res<ChunkLookup>,
    mut commands: Commands,
    map_gen: Res<MeshGenerator>,
) {
    let offset = trigger.offset();
    player.translation = Vec3::splat(16.);

    for material in &mats {
        let Some(matt) = materials.get_mut(material.id()) else {
            continue;
        };
        matt.data = map_gen.dummy_image();
    }

    for (id, data) in chunks.iter() {
        let next = *id + **trigger;
        let Some(next) = lookup.get(&next) else {
            // error!("no next");
            continue;
        };
        let Ok(material) = mats.get(next) else {
            // error!("Failed to get chunk");
            continue;
        };
        let Some(chunk) = chunk_data.get(data.data.id()) else {
            // error!("Chunk data missing");
            continue;
        };
        let Some(matt) = materials.get_mut(material.id()) else {
            // error!("Material missing");
            continue;
        };
        if let Some(image) = &chunk.images {
            matt.data = image.clone();
        }
        commands.entity(next).insert((data.clone(), *id));
    }
}

#[derive(Event, Deref)]
struct MoveWorld(IVec3);

impl MoveWorld {
    fn offset(&self) -> Vec3 {
        (self.0 * CHUNK_SIZE as i32).as_vec3()
    }
}
