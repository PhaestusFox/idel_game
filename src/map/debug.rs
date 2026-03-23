use std::f32::consts::PI;

use bevy::{
    feathers::{controls::*, theme::ThemedText},
    prelude::*,
    ui::Checked,
    ui_widgets::{
        Activate, SliderPrecision, SliderRange, SliderStep, SliderValue, ValueChange, observe,
    },
};

use crate::map::{
    ChunkGenerator, ChunkId,
    map_gen::biomes::{DebugBiome, DebugBiomeType},
};

pub struct MapDebugConsolePlugin;

impl Plugin for MapDebugConsolePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct MapDebugConsole;

pub fn toggle_console(
    on: On<ValueChange<bool>>,
    console: Query<Entity, With<MapDebugConsole>>,
    mut commands: Commands,
) {
    if on.value {
        commands.entity(on.source).insert(Checked);
        open_console(commands);
    } else {
        commands.entity(on.source).remove::<Checked>();
        console.iter().for_each(|console| {
            info!("Console closed");
            commands.entity(console).despawn();
        });
    }
}

fn open_console(mut commands: Commands) {
    commands.spawn((
        MapDebugConsole,
        Node {
            width: Val::Px(300.),
            height: Val::Px(200.),
            right: Val::Px(0.),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        Name::new("Map Debug Console"),
        BackgroundColor(Color::WHITE.darker(0.3)),
        crate::ui::Anchor,
        children![
            (button(
                ButtonProps {
                    variant: ButtonVariant::Primary,
                    ..Default::default()
                },
                observe(regenerate_chunks),
                Spawn((Text::new("Regenerate Chunks"), ThemedText)),
            )),
            (
                Node::DEFAULT,
                children![
                    slider(
                        SliderProps {
                            value: super::MAP_SIZE_X as f32,
                            min: 0.,
                            max: 20.
                        },
                        (
                            Text::new("X:"),
                            ThemedText,
                            Set::X,
                            SliderPrecision(0),
                            observe(set_map_descriptor)
                        ),
                    ),
                    slider(
                        SliderProps {
                            value: super::MAP_DEPTH as f32,
                            min: 0.,
                            max: 5.
                        },
                        (
                            SliderPrecision(0),
                            Text::new("Y:"),
                            ThemedText,
                            Set::Y,
                            observe(set_map_descriptor)
                        )
                    ),
                    slider(
                        SliderProps {
                            value: super::MAP_SIZE_Z as f32,
                            min: 0.,
                            max: 20.
                        },
                        (
                            Text::new("Z:"),
                            ThemedText,
                            SliderPrecision(0),
                            Set::Z,
                            observe(set_map_descriptor)
                        )
                    )
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                children![
                    slider(
                        SliderProps {
                            value: 6.,
                            min: 1.,
                            max: 10.
                        },
                        (
                            Text::new("Octaves:"),
                            ThemedText,
                            Set::Ocataves,
                            SliderPrecision(0),
                            observe(set_map_descriptor)
                        ),
                    ),
                    slider(
                        SliderProps {
                            value: 1.,
                            min: 0.01,
                            max: 5.
                        },
                        (
                            SliderPrecision(2),
                            Text::new("Frequency:"),
                            ThemedText,
                            Set::Frequency,
                            observe(set_map_descriptor)
                        )
                    ),
                    slider(
                        SliderProps {
                            value: (PI * 2.) / 3., // 2.0 * PI / 3.0
                            min: 0.5,
                            max: 4.
                        },
                        (
                            Text::new("Lacunarity:"),
                            ThemedText,
                            Set::Lacunarity,
                            observe(set_map_descriptor)
                        )
                    ),
                    slider(
                        SliderProps {
                            value: 0.5,
                            min: -2.,
                            max: 2.0
                        },
                        (
                            Text::new("Persistence:"),
                            ThemedText,
                            SliderPrecision(2),
                            Set::Persistence,
                            observe(set_map_descriptor)
                        )
                    )
                ],
            ),
            (
                Node::DEFAULT,
                children![
                    button(
                        ButtonProps::default(),
                        (super::DebugBiomeType::Height, observe(set_debug_biome)),
                        Spawn((Text::new("Height"), ThemedText))
                    ),
                    button(
                        ButtonProps::default(),
                        (super::DebugBiomeType::Rainfall, observe(set_debug_biome)),
                        Spawn((Text::new("Rainfall"), ThemedText))
                    ),
                    button(
                        ButtonProps::default(),
                        (super::DebugBiomeType::Fertility, observe(set_debug_biome)),
                        Spawn((Text::new("Fertility"), ThemedText))
                    ),
                    button(
                        ButtonProps::default(),
                        (
                            super::DebugBiomeType::GroundHeight2,
                            observe(set_debug_biome)
                        ),
                        Spawn((Text::new("GroundHeight2"), ThemedText))
                    )
                ]
            ),
            slider(
                SliderProps {
                    value: 1.,
                    min: 1.,
                    max: 10.,
                },
                (
                    Text::new("Scale:"),
                    ThemedText,
                    SliderPrecision(0),
                    Set::Scale,
                    observe(set_debug_scale)
                )
            )
        ],
    ));
}

fn regenerate_chunks(
    _: On<Activate>,
    mut chunk_think: ResMut<ChunkGenerator>,
    map: Res<super::MapDescriptor>,
    chunks: Query<(&ChunkId, Entity), With<super::chunk::Chunk>>,
    mut commands: Commands,
) {
    let mut kill = chunks
        .into_iter()
        .map(|(k, v)| (*k, v))
        .collect::<bevy::platform::collections::HashMap<_, _>>();
    for x in -map.world_size.x..=map.world_size.x {
        for y in -map.world_size.y..=map.world_size.y {
            for z in -map.world_size.z..=map.world_size.z {
                let chunk_id = ChunkId::new(x, y, z);
                kill.remove(&chunk_id);
                chunk_think.que(*chunk_id);
            }
        }
    }
    for (_, entity) in kill {
        commands.entity(entity).despawn();
    }
}

#[derive(Component, Debug)]
enum Set {
    X,
    Y,
    Z,
    Ocataves,
    Frequency,
    Lacunarity,
    Persistence,
    Scale,
}
/*

    pub const DEFAULT_OCTAVE_COUNT: usize = 6;
    pub const DEFAULT_FREQUENCY: f64 = 1.0;
    pub const DEFAULT_LACUNARITY: f64 = core::f64::consts::PI * 2.0 / 3.0;
    pub const DEFAULT_PERSISTENCE: f64 = 0.5;
    pub const MAX_OCTAVES: usize = 32;

    /// Total number of frequency octaves to generate the noise with.
    ///
    /// The number of octaves control the _amount of detail_ in the noise
    /// function. Adding more octaves increases the detail, with the drawback
    /// of increasing the calculation time.
    pub octaves: usize,

    /// The number of cycles per unit length that the noise function outputs.
    pub frequency: f64,

    /// A multiplier that determines how quickly the frequency increases for
    /// each successive octave in the noise function.
    ///
    /// The frequency of each successive octave is equal to the product of the
    /// previous octave's frequency and the lacunarity value.
    ///
    /// A lacunarity of 2.0 results in the frequency doubling every octave. For
    /// almost all cases, 2.0 is a good value to use.
    pub lacunarity: f64,

    /// A multiplier that determines how quickly the amplitudes diminish for
    /// each successive octave in the noise function.
    ///
    /// The amplitude of each successive octave is equal to the product of the
    /// previous octave's amplitude and the persistence value. Increasing the
    /// persistence produces "rougher" noise.
    pub persistence: f64,
*/

fn set_map_descriptor(
    change: On<ValueChange<f32>>,
    mut map: ResMut<super::MapDescriptor>,
    mut generator: ResMut<ChunkGenerator>,
    sliders: Query<&Set>,
    mut commands: Commands,
) {
    let Ok(set) = sliders.get(change.source) else {
        warn!("Value change event from unknown source");
        return;
    };
    commands
        .entity(change.source)
        .insert(SliderValue(change.value));
    println!("Setting map descriptor {:?} to {}", set, change.value);
    match set {
        Set::X => map.world_size.x = change.value as i32,
        Set::Y => map.world_size.y = change.value as i32,
        Set::Z => map.world_size.z = change.value as i32,
        Set::Ocataves => generator.set_octaves(change.value as usize),
        Set::Frequency => generator.set_frequency(change.value as f64),
        Set::Lacunarity => generator.set_lacunarity(change.value as f64),
        Set::Persistence => generator.set_persistence(change.value as f64),
        _ => {}
    }
}

fn set_debug_biome(
    change: On<Activate>,
    map: ResMut<super::ChunkGenerator>,
    button: Query<&DebugBiomeType>,
) {
    let Ok(param) = button.get(change.entity) else {
        warn!("Activate event from unknown source");
        return;
    };
    println!("Setting debug biome to {:?}", param);
    let mut map = map.map.write().unwrap();
    let biomes = map.biomes_mut();

    for biome in biomes.iter_mut() {
        if biome.name() != "Debug" {
            continue;
        }
        let b = biome.as_any_mut().downcast_mut::<DebugBiome>().unwrap();
        b.param = *param;
    }
}

fn set_debug_scale(
    change: On<ValueChange<f32>>,
    map: ResMut<super::ChunkGenerator>,
    sliders: Query<&Set>,
    mut commands: Commands,
) {
    let Ok(set) = sliders.get(change.source) else {
        warn!("Value change event from unknown source");
        return;
    };
    commands
        .entity(change.source)
        .insert(SliderValue(change.value));
    let mut map = map.map.write().unwrap();
    let biomes = map.biomes_mut();
    for biome in biomes.iter_mut() {
        if biome.name() != "Debug" {
            continue;
        }
        let b = biome.as_any_mut().downcast_mut::<DebugBiome>().unwrap();
        b.scale = change.value as u32;
    }
}
