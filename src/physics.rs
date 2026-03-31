use bevy::prelude::*;

use crate::map::{CHUNK_SIZE, Chunk, ChunkData, ChunkId, ChunkLookup};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Gravity>();
        app.insert_resource(Time::<Fixed>::from_hz(30.))
            .configure_sets(
                FixedUpdate,
                PhysicsStep::ApplyVelocity.after(PhysicsStep::UpdateVelocity),
            );

        // pre update velocity
        app.add_systems(
            FixedUpdate,
            update_grounded.before(PhysicsStep::UpdateVelocity),
        );

        // Updates to velocity
        app.add_systems(FixedUpdate, add_gravity.in_set(PhysicsStep::UpdateVelocity))
            .add_systems(
                FixedUpdate,
                apply_resistance
                    .in_set(PhysicsStep::UpdateVelocity)
                    .after(add_gravity),
            );
        // using velocity
        app.add_systems(
            FixedUpdate,
            apply_velocity.in_set(PhysicsStep::ApplyVelocity),
        );

        #[cfg(debug_assertions)]
        app.register_type::<Velocity>().register_type::<Grounded>();
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PhysicsStep {
    UpdateVelocity,
    ApplyVelocity,
}

/// apply gravity
fn add_gravity(
    mut objects: Query<(&mut Velocity, &Grounded), Without<Weightless>>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    let g = gravity.0 * time.delta_secs();
    for (mut velocity, Grounded(on_ground)) in &mut objects {
        if *on_ground {
            continue;
        }
        velocity.0 -= g;
    }
}

// apply velocity to entities
fn apply_velocity(
    mut objects: Query<(&mut Transform, &Velocity), Without<Weightless>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut objects {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

fn apply_resistance(mut objects: Query<(&mut Velocity, &Grounded), Without<Weightless>>) {
    for (mut velocity, Grounded(on_ground)) in &mut objects {
        if *on_ground {
            velocity.0 *= 0.9;
            velocity.0.y = velocity.y.max(0.);
        } else {
            velocity.0 *= 0.99;
        }
    }
}

#[derive(Resource)]
pub struct Gravity(Vec3);

impl Gravity {
    pub const GRAVITY_STRENGTH: f32 = 9.81 * 4.;
}

impl FromWorld for Gravity {
    fn from_world(_world: &mut World) -> Self {
        Self(Vec3::Y * Self::GRAVITY_STRENGTH)
    }
}

#[derive(Component, Default, Deref, DerefMut, Reflect, Clone, Copy)]
pub struct Grounded(bool);

impl core::ops::BitOr<bool> for Grounded {
    type Output = bool;

    fn bitor(self, rhs: bool) -> Self::Output {
        self.0 | rhs
    }
}

impl core::ops::BitOr<bool> for &Grounded {
    type Output = bool;

    fn bitor(self, rhs: bool) -> Self::Output {
        self.0 | rhs
    }
}

impl core::ops::BitOr<Grounded> for bool {
    type Output = bool;
    fn bitor(self, rhs: Grounded) -> Self::Output {
        self | rhs.0
    }
}
impl core::ops::BitOr<&Grounded> for bool {
    type Output = bool;
    fn bitor(self, rhs: &Grounded) -> Self::Output {
        self | rhs.0
    }
}

impl core::ops::BitAnd<bool> for Grounded {
    type Output = bool;

    fn bitand(self, rhs: bool) -> Self::Output {
        self.0 & rhs
    }
}

impl core::ops::BitAnd<bool> for &Grounded {
    type Output = bool;

    fn bitand(self, rhs: bool) -> Self::Output {
        self.0 & rhs
    }
}

impl core::ops::BitAnd<Grounded> for bool {
    type Output = bool;
    fn bitand(self, rhs: Grounded) -> Self::Output {
        self & rhs.0
    }
}
impl core::ops::BitAnd<&Grounded> for bool {
    type Output = bool;
    fn bitand(self, rhs: &Grounded) -> Self::Output {
        self & rhs.0
    }
}

#[derive(Component)]
pub struct Weightless;

#[derive(Component, Default, Reflect, Deref, DerefMut)]
#[require(Grounded = Grounded(false))]
pub struct Velocity(Vec3);

fn update_grounded(
    mut objects: Query<(&mut Grounded, &Transform)>,
    map: Res<ChunkLookup>,
    data: Res<Assets<ChunkData>>,
    chunks: Query<&Chunk>,
    world_offset: Res<ChunkId>,
) {
    for (mut grounded, transform) in &mut objects {
        let pos = transform.translation.floor() - Vec3::Y + Vec3::splat(CHUNK_SIZE as f32 * 0.5);
        let chunk_id = ChunkId::from_translation(pos) + *world_offset;
        let Some(chunk_entity) = map.get(&chunk_id) else {
            **grounded = true;
            continue;
        };
        let Ok(chunk) = chunks.get(chunk_entity) else {
            **grounded = true;
            continue;
        };
        let Some(chunk_data) = data.get(&chunk.data) else {
            **grounded = true;
            continue;
        };
        let foot = (pos).as_ivec3().rem_euclid(IVec3::splat(CHUNK_SIZE as i32));
        let block = chunk_data.get_block(foot.x as u8, foot.y as u8, foot.z as u8);
        **grounded = block.is_solid();
    }
}
