use std::time;

use bevy::{
    light::{FogVolume, VolumetricLight},
    pbr::DefaultOpaqueRendererMethod,
};

use super::*;
pub fn plugin(app: &mut App) {
    app.insert_resource(DefaultOpaqueRendererMethod::deferred())
        .insert_resource(GlobalAmbientLight::NONE)
        .insert_resource(ClearColor(Color::BLACK));
    app.add_systems(Startup, spawn_sun);
    app.add_systems(Update, day_night);
}

fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            // lux::RAW_SUNLIGHT is recommended for use with this feature, since
            // other values approximate sunlight *post-scattering* in various
            // conditions. RAW_SUNLIGHT in comparison is the illuminance of the
            // sun unfiltered by the atmosphere, so it is the proper input for
            // sunlight to be filtered by the atmosphere.
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(1.0, 0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
    ));

    // commands.spawn((
    //     FogVolume::default(),
    //     Transform::from_scale(Vec3::splat(CHUNK_SIZE as f32 * 2.)).with_translation(Vec3::Y * 0.5),
    // ));
}

const DAY_LENGTH: f32 = 10.; //In seconds
fn day_night(mut suns: Query<&mut Transform, With<DirectionalLight>>, time: Res<Time>) {
    for mut sun in &mut suns {
        sun.rotate_x(time.delta_secs() * core::f32::consts::PI * 2. / DAY_LENGTH); 
    }
}
