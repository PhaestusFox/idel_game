use super::*;

use bevy::prelude::*;

pub struct BiomeDebugPlugin;

impl Plugin for BiomeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeSeeker>();

        app.add_systems(
            Update,
            seek_biome_info.run_if(|bs: Res<BiomeSeeker>| bs.ray_cast_biome_info),
        );
    }
}

#[derive(Resource)]
pub struct BiomeSeeker {
    ray_cast_biome_info: bool,
}

impl FromWorld for BiomeSeeker {
    fn from_world(_world: &mut World) -> Self {
        Self {
            ray_cast_biome_info: false,
        }
    }
}

pub fn toggle_biome_seeker(
    check: On<ValueChange<bool>>,
    mut biome_seeker: ResMut<BiomeSeeker>,
    mut commands: Commands,
    open: Query<(Entity, &BiomeInfoText)>,
    console: Single<Entity, With<DebugConsole>>,
) {
    if check.value {
        commands.entity(check.source).insert(Checked);
        spawn_biome_info(commands, *console);
    } else {
        commands.entity(check.source).remove::<Checked>();
        open.iter().for_each(|(entity, text)| {
            if BiomeInfoText::Root != *text {
                commands.entity(entity).despawn();
            }
        });
    }
    biome_seeker.ray_cast_biome_info = check.value;
}

fn spawn_biome_info(mut commands: Commands, console: Entity) {
    commands.entity(console).with_child((
        BiomeInfoText::Root,
        Node {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        children![
            (Text::new("Biome Info:"), BiomeInfoText::Main),
            (Text::new("Secondary Biome:"), BiomeInfoText::Secondary,),
            (Text::new("Third Biome:"), BiomeInfoText::Third,),
        ],
    ));
}

#[derive(Component, PartialEq, Eq)]
pub enum BiomeInfoText {
    Root,
    Main,
    Secondary,
    Third,
}

use crate::{
    map::{CHUNK_OFFSET, Map},
    player::Raycast,
};

macro_rules! biome_info_template {
    () => {
        r#"{}: {} ({}): {:.0}%"#
    };
}

fn seek_biome_info(
    ray: Raycast,
    input: Res<ButtonInput<MouseButton>>,
    map: Map,
    descriptor: Res<crate::map::ChunkGenerator>,
    mut text: Query<(&mut Text, &BiomeInfoText)>,
) {
    // only run if middle click
    if !input.just_pressed(MouseButton::Middle) {
        return;
    }
    // only run if we hit a block
    let Some(block) = ray.get_player_hit() else {
        warn!("No block hit");
        return;
    };
    let block = block + CHUNK_OFFSET.as_ivec3();
    let map = descriptor.map.read().unwrap();
    let out = map.calculate_biomes(block.x, block.z);
    let biomes = map.biomes();
    let strengths =
        out.map(|(index, _)| biomes[index].strength(IVec2::new(block.x, block.z), &map));
    let scaled = vec3(strengths[0], strengths[1], strengths[2]).normalize();
    let correct = scaled[0] + scaled[1] + scaled[2];
    let total_strength = (scaled / correct).to_array();

    for (mut text, biome_info) in &mut text {
        let (biome_id, ident) = match biome_info {
            BiomeInfoText::Main => (0, "Main Biome"),
            BiomeInfoText::Secondary => (1, "Secondary Biome"),
            BiomeInfoText::Third => (2, "Third Biome"),
            BiomeInfoText::Root => continue,
        };
        let (index, priorty) = out[biome_id];
        if priorty == 0 {
            text.0 = format!("{}: None", ident);
            continue;
        }
        let biome = &biomes[index];
        let name = biome.name();
        let strength = total_strength[biome_id];

        text.0 = format!(
            biome_info_template!(),
            ident,
            name,
            priorty,
            strength * 100.
        );
    }
}
