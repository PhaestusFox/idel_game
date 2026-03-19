use bevy::{
    feathers::{controls::*, theme::ThemedText},
    prelude::*,
    ui::Checked,
    ui_widgets::{
        Activate, SliderPrecision, SliderRange, SliderStep, SliderValue, ValueChange, observe,
    },
};

use crate::map::{ChunkGenerator, ChunkId};

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
                        (Set::X, SliderPrecision(0), observe(set_map_descriptor)),
                    ),
                    slider(
                        SliderProps {
                            value: super::MAP_DEPTH as f32,
                            min: 0.,
                            max: 5.
                        },
                        (SliderPrecision(0), Set::Y, observe(set_map_descriptor))
                    ),
                    slider(
                        SliderProps {
                            value: super::MAP_SIZE_Z as f32,
                            min: 0.,
                            max: 20.
                        },
                        (SliderPrecision(0), Set::Z, observe(set_map_descriptor))
                    )
                ],
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
}

fn set_map_descriptor(
    change: On<ValueChange<f32>>,
    mut map: ResMut<super::MapDescriptor>,
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
    }
}
