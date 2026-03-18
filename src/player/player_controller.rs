use crate::{
    map::{Block, Map},
    physics::{self, Grounded, PhysicsStep, Velocity},
};

use super::*;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            move_player
                .in_set(PlayerMovement)
                .run_if(in_state(MoveMode::Walk)),
        )
        .add_systems(PreUpdate, jump.in_set(PhysicsStep::UpdateVelocity));
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    move_bindings: Res<MoveBindings>,
    settings: Res<FlyCameraSettings>,
    mut player: Single<&mut Transform, With<PlayerEntity>>,
    map: Map,
    chunks: Query<&Chunk>,
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
    // TODO: prevent corner clipping
    let mut move_dir = (right * input.x + forward * input.y).normalize_or_zero();
    if keyboard_input.pressed(move_bindings.boost) {
        move_dir *= settings.boost_multiplier;
    }
    let mut delta = move_dir * settings.move_speed * time.delta_secs();
    let mut x_check = player.translation;
    x_check.x += delta.x + delta.x.signum() * 0.1;
    let r = map
        .get_block(x_check, chunks)
        .map(|b| b.is_solid())
        .unwrap_or(false)
        | map
            .get_block(x_check + Vec3::Y, chunks)
            .map(|b| b.is_solid())
            .unwrap_or(false);
    if r {
        delta.x = 0.;
    }
    let mut z_check = player.translation;
    z_check.z += delta.z + delta.z.signum() * 0.1;
    let f = map
        .get_block(z_check, chunks)
        .map(|b| b.is_solid())
        .unwrap_or(false)
        | map
            .get_block(z_check + Vec3::Y, chunks)
            .map(|b| b.is_solid())
            .unwrap_or(false);
    if f {
        delta.z = 0.;
    }

    player.translation += delta;
}

fn jump(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Single<(&mut Transform, &mut Velocity, &Grounded), With<PlayerEntity>>,
    map: Map,
    chunks: Query<&Chunk>,
) {
    let (mut transform, mut velocity, grounded) = player.into_inner();
    if keyboard_input.just_pressed(KeyCode::Space) & grounded
        && map
            .get_block(transform.translation + Vec3::new(0., 2.1, 0.), chunks)
            .map_or(true, |b| !b.is_solid())
    {
        velocity.y += physics::Gravity::GRAVITY_STRENGTH * 0.25;
    }
}
