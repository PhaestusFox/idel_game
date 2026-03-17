use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec3::Y * 9.8));
        app.insert_resource(Time::<Fixed>::from_hz(30.))
            .configure_sets(
                FixedUpdate,
                PhysicsStep::ApplyVelocity.after(PhysicsStep::UpdateVelocity),
            );

        app.add_systems(FixedUpdate, add_gravity.in_set(PhysicsStep::UpdateVelocity))
            .add_systems(
                FixedUpdate,
                air_resistance
                    .in_set(PhysicsStep::UpdateVelocity)
                    .after(add_gravity),
            );

        app.add_systems(
            FixedUpdate,
            apply_velocity.in_set(PhysicsStep::ApplyVelocity),
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsStep {
    UpdateVelocity,
    ApplyVelocity,
}

/// apply gravity
fn add_gravity(
    mut objects: Query<&mut Velocity, Without<Weightless>>,
    gravity: Res<Gravity>,
    time: Res<Time>,
) {
    let g = gravity.0 * time.delta_secs();
    for mut velocity in &mut objects {
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

fn air_resistance(mut objects: Query<&mut Velocity, Without<Weightless>>) {
    for mut velocity in &mut objects {
        velocity.0 *= 0.99;
    }
}

#[derive(Resource)]
pub struct Gravity(Vec3);

#[derive(Component)]
pub struct Weightless;

#[derive(Component, Default)]
pub struct Velocity(Vec3);
