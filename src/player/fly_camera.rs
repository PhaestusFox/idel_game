use crate::physics::Weightless;

use super::*;

use bevy::prelude::*;

pub struct FlyPlugin;

impl Plugin for FlyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlyCameraSettings>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                PreUpdate,
                (move_player
                    .after(player_look)
                    .in_set(PlayerMovement)
                    .run_if(in_state(MoveMode::Fly)),),
            );

        // Physics
        app.add_systems(
            OnEnter(MoveMode::Fly),
            |player: Single<Entity, With<PlayerEntity>>, mut commands: Commands| {
                commands.entity(*player).insert(Weightless);
            },
        )
        .add_systems(
            OnExit(MoveMode::Fly),
            |player: Single<Entity, With<PlayerEntity>>, mut commands: Commands| {
                commands.entity(*player).remove::<Weightless>();
            },
        );
    }
}

#[derive(Resource)]
pub struct FlyCameraSettings {
    pub move_speed: f32,
    pub boost_multiplier: f32,
}

impl Default for FlyCameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 32.,
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
