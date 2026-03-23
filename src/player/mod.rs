use bevy::{
    anti_alias::fxaa::Fxaa,
    input::{common_conditions::input_just_pressed, mouse::AccumulatedMouseMotion},
    light::{FogVolume, VolumetricFog, VolumetricLight},
    pbr::{Atmosphere, AtmosphereSettings, DefaultOpaqueRendererMethod, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};

use crate::{
    GameState,
    map::{CHUNK_SIZE, Chunk, ChunkBlock, ChunkData, ChunkGenerator, ChunkId, ChunkLookup, LoD},
    physics::Weightless,
    rendering::CustomMaterial,
};

mod fly_camera;
pub use fly_camera::FlyCameraSettings;

mod player_controller;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(player_controller::PlayerControllerPlugin);
        app.configure_sets(
            PreUpdate,
            PlayerMovement.run_if(in_state(GameState::Playing)),
        );

        app.init_state::<MoveMode>();

        app.init_resource::<MoveBindings>()
            .init_resource::<CameraSettings>()
            .add_plugins(fly_camera::FlyPlugin)
            .add_systems(OnEnter(GameState::Playing), hide_cursor)
            .add_systems(OnExit(GameState::Playing), show_cursor);

        app.add_systems(PreUpdate, player_look.in_set(PlayerMovement));

        app.add_observer(move_the_universe_not_the_ship)
            .add_systems(First, detect_chunk_transition)
            .init_resource::<ChunkId>();
        app.add_systems(
            Update,
            update_visible_chunks.run_if(resource_changed::<ChunkId>),
        );
        app.init_resource::<RenderDistance>();
    }
}

#[derive(Component)]
#[require(ChunkId)]
pub struct PlayerEntity;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Name::new("Player"),
            Transform::from_xyz(0.0, 10.0, 0.0),
            Weightless,
            Visibility::Visible,
            crate::physics::Velocity::default(),
            PlayerEntity,
        ))
        .with_child((
            Name::new("PlayerCamera"),
            Camera3d::default(),
            Transform::from_xyz(0.0, 1.75, 0.).looking_at(Vec3::ZERO, Vec3::Y),
            Atmosphere {
                top_radius: 6_560_000.,
                bottom_radius: 6_360_000.,
                ground_albedo: Vec3::new(0.1, 0.3, 0.1),
                medium: asset_server.add(ScatteringMedium::earthlike(128, 128)),
            },
            AtmosphereSettings::default(),
            Bloom::NATURAL,
            VolumetricFog {
                ambient_intensity: 0.0,
                ..default()
            },
            Msaa::Off,
            bevy::camera::Exposure { ev100: 12.0 },
            Fxaa::default(),
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

pub fn show_cursor(mut windows: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    windows.visible = true;
    windows.grab_mode = bevy::window::CursorGrabMode::None;
}

pub fn hide_cursor(mut windows: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    windows.visible = false;
    windows.grab_mode = bevy::window::CursorGrabMode::Locked;
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

pub fn detect_chunk_transition(
    player: Single<&Transform, With<PlayerEntity>>,
    mut commands: Commands,
    mut generator: ResMut<ChunkGenerator>,
) {
    generator.set_dirty();
    let pos = ChunkId::from_translation(player.translation);
    if pos.abs().max_element() > 1 {
        commands.trigger(MoveWorld(pos.clamp(IVec3::NEG_ONE, IVec3::ONE)));
    }
}

fn move_the_universe_not_the_ship(
    trigger: On<MoveWorld>,
    mut world_offset: ResMut<ChunkId>,
    mut player: Single<&mut Transform, With<PlayerEntity>>,
    mut transforms: Query<&mut Transform, (With<ChunkBlock>, Without<PlayerEntity>)>,
) {
    player.translation -= trigger.offset();
    for mut block in &mut transforms {
        block.translation -= trigger.offset();
    }
    *world_offset += trigger.0;
}

#[derive(Event, Deref)]
struct MoveWorld(IVec3);

impl MoveWorld {
    fn offset(&self) -> Vec3 {
        (self.0 * CHUNK_SIZE as i32).as_vec3()
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MoveMode {
    #[default]
    Fly,
    Walk,
}

#[derive(Resource)]
pub struct CameraSettings {
    pub look_sensitivity: f32,
    pub invert_look_y: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.05,
            invert_look_y: false,
        }
    }
}

fn player_look(
    mouse_movement: Res<AccumulatedMouseMotion>,
    mut player: Single<(&mut Transform, &Children), With<PlayerEntity>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<PlayerEntity>)>,
    settings: Res<CameraSettings>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
) {
    if matches!(cursor.grab_mode, bevy::window::CursorGrabMode::None) {
        return;
    }
    let mut delta = mouse_movement.delta * settings.look_sensitivity;
    if settings.invert_look_y {
        delta.y = -delta.y;
    }
    let Ok(mut camera) = camera.get_mut(player.1[0]) else {
        error!("Player camera entity was despawned");
        return;
    };
    let (_, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    // let (mut yaw, _, _) = player.0.rotation.to_euler(EulerRot::YXZ);
    // yaw -= delta.x.to_radians();
    pitch -= delta.y.to_radians();
    pitch = pitch.clamp(-core::f32::consts::FRAC_PI_2, core::f32::consts::FRAC_PI_2);
    camera.rotation = Quat::from_euler(EulerRot::YXZ, 0., pitch, 0.0);
    player.0.rotate_y(-delta.x.to_radians());
}

#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Debug)]
struct PlayerMovement;

/// The distance at which chunks will be set visible.
#[derive(Debug, Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct RenderDistance(u32);

impl Default for RenderDistance {
    fn default() -> Self {
        Self(10)
    }
}

impl PartialEq<u32> for RenderDistance {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl std::cmp::PartialOrd<u32> for RenderDistance {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

fn update_visible_chunks(
    offset: Res<ChunkId>,
    view_distance: Res<RenderDistance>,
    chunks: Query<(&ChunkId, &mut Visibility)>,
) {
    for (chunk_id, mut visibility) in chunks {
        let distance = chunk_id.chebyshev_distance(**offset);
        if *view_distance < distance {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}
