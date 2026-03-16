use std::sync::Arc;
use std::sync::RwLock;

use bevy::{platform::collections::HashMap, prelude::*, tasks::Task};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<chunk::ChunkData>();
        app.init_resource::<MeshGenerator>();
        app.add_systems(Startup, spawn_test_chunk);
        app.add_systems(Update, update_mesh_generator);
    }
}

mod chunk;
mod map_gen;
pub use chunk::*;
use indexmap::IndexSet;

use crate::rendering::CustomMaterial;

fn spawn_test_chunk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut chunk_manager: ResMut<MeshGenerator>,
) {
    for x in -32..=32 {
        for y in -4..=4 {
            for z in -32..=32 {
                chunk_manager.que(IVec3::new(x, y, z));
            }
        }
    }
}

#[derive(Resource)]
struct MeshGenerator {
    main_mesh: Handle<Mesh>,
    meshs: HashMap<LoD, Handle<Mesh>>,
    dummy_image: Handle<Image>,
    tasks: HashMap<Entity, Task<(ChunkData, Image)>>,
    new_tasks: HashMap<Entity, Task<(ChunkData, Image)>>,
    root: Entity,
    dirty: bool,
    max_chunk_tasks: usize,
    que: IndexSet<IVec3>,
    world: Arc<RwLock<map_gen::MapDescriptor>>,
}

impl FromWorld for MeshGenerator {
    fn from_world(world: &mut World) -> Self {
        let root = world
            .spawn((
                Name::new("Chunks"),
                Transform::IDENTITY,
                Visibility::Visible,
            ))
            .id();
        let asset_server = world.resource::<AssetServer>();
        let mut lods = HashMap::new();
        for lod in [LoD::LOD2, LoD::LOD4, LoD::LOD8, LoD::LOD16] {
            lods.insert(lod, asset_server.add(make_baked_mesh_lod(lod)));
        }

        Self {
            root,
            main_mesh: asset_server.add(make_baked_mesh()),
            meshs: lods,
            tasks: HashMap::default(),
            new_tasks: HashMap::default(),
            que: IndexSet::default(),
            dirty: false,
            dummy_image: asset_server.add(ChunkData::dummy_image()),
            max_chunk_tasks: 100,
            world: Arc::new(RwLock::new(map_gen::MapDescriptor::default())),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum LoD {
    LOD1,
    LOD2,
    LOD4,
    LOD8,
    LOD16,
}
impl LoD {
    fn step(&self) -> f32 {
        match self {
            LoD::LOD1 => 1.,
            LoD::LOD2 => 2.,
            LoD::LOD4 => 4.,
            LoD::LOD8 => 8.,
            LoD::LOD16 => 16.,
        }
    }
}

impl MeshGenerator {
    fn generate(
        &mut self,
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        player_pos: Vec3,
    ) {
        if self.que.is_empty() {
            return;
        }
        let pool = bevy::tasks::AsyncComputeTaskPool::get();
        self.sort_que(player_pos);
        let que = self
            .que
            .iter()
            .rev()
            .take(pool.thread_num() * 4)
            .cloned()
            .collect::<Vec<_>>();
        let colors = asset_server.load("colors.png");
        println!("Generating {} chunks", que.len());
        for chunk in que {
            self.que.swap_remove(&chunk);
            let descriptor = self.world.clone();
            let task = pool.spawn(async move {
                let descriptor = descriptor.read().unwrap();
                ChunkData::generate(chunk, &descriptor)
            });
            let dis = chunk.as_vec3().distance(Vec3::ZERO);
            let lod = if dis < 16. {
                LoD::LOD1
            } else if dis < 32. {
                LoD::LOD2
            } else if dis < 48. {
                LoD::LOD4
            } else if dis < 64. {
                LoD::LOD8
            } else {
                LoD::LOD16
            };
            let mesh = self.get_mesh(lod);
            let id = commands
                .spawn((
                    Name::new(format!("Chunk ({})", chunk)),
                    Mesh3d(mesh),
                    MeshMaterial3d(asset_server.add(crate::rendering::CustomMaterial {
                        lod: lod.step(),
                        color_texture: Some(colors.clone()),
                        alpha_mode: AlphaMode::Opaque,
                        chunk_offset: Vec3::ZERO,
                        data: self.dummy_image.clone(),
                    })),
                    Transform::from_translation(
                        (chunk * CHUNK_SIZE as i32).as_vec3()
                            + Vec3::splat(CHUNK_SIZE as f32 * 0.5),
                    )
                    .with_scale(Vec3::splat(CHUNK_SIZE as f32 * 0.5)),
                    ChildOf(self.root),
                    ChunkId::new(chunk),
                ))
                .id();
            self.tasks.insert(id, task);
        }
    }

    fn que(&mut self, pos: IVec3) {
        if self.que.insert(pos) {
            self.dirty = true;
        }
    }

    fn sort_que(&mut self, player_pos: Vec3) {
        if !self.dirty {
            return;
        }
        let center = (player_pos / CHUNK_SIZE as f32).as_ivec3();
        self.que.sort_by(|a, b| {
            b.manhattan_distance(center)
                .cmp(&a.manhattan_distance(center))
        });
        self.dirty = false;
    }

    fn get_mesh(&self, lod: LoD) -> Handle<Mesh> {
        self.meshs.get(&lod).unwrap_or(&self.main_mesh).clone()
    }
}

fn update_mesh_generator(
    mut mesh_generator: ResMut<MeshGenerator>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    chunks: Query<(&ChunkId, &MeshMaterial3d<CustomMaterial>)>,
    mut mashes: ResMut<Assets<CustomMaterial>>,
    player: Single<&Transform, With<crate::player::PlayerEntity>>,
    mut done: Local<bool>,
) {
    let mut keep = HashMap::default();
    for (id, task) in mesh_generator.tasks.drain() {
        if task.is_finished() {
            let Ok((chunk_id, material)) = chunks.get(id) else {
                error!("Chunk entity was despawned before mesh generation finished");
                continue;
            };
            let (data, image) = bevy::tasks::block_on(task.cancel()).expect("checked was finished");
            let chunk = Chunk {
                is_empty: data.is_empty,
                data: asset_server.add(data),
            };
            let Some(material) = mashes.get_mut(material.id()) else {
                error!("CustomMaterial asset was removed before mesh generation finished");
                continue;
            };
            material.chunk_offset = chunk_id.offset();
            if !chunk.is_empty {
                material.data = asset_server.add(image);
            } else {
                commands.entity(id).insert(Visibility::Hidden);
            }
            commands.entity(id).insert(chunk);
        } else {
            keep.insert(id, task);
        }
    }
    mesh_generator.tasks = keep;
    if mesh_generator.tasks.is_empty() && !*done && mesh_generator.que.is_empty() {
        println!("All chunks generated");
        *done = true;
    }
    if !mesh_generator.que.is_empty() {
        *done = false;
    }
    if mesh_generator.tasks.len() > mesh_generator.max_chunk_tasks {
        return;
    }
    mesh_generator.generate(&mut commands, asset_server, player.translation);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Block {
    Void = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    BedRock = 4,
    Snow = 5,
    Sand = 6,
    Other(u8),
}

impl Block {
    fn is_solid(&self) -> bool {
        matches!(self, Block::Void) == false
    }
    fn color(&self) -> Color {
        use bevy::color::palettes::css::*;
        use bevy::color::palettes::tailwind::*;
        match self {
            Block::Void => return Color::linear_rgba(0., 0., 0., 0.),
            Block::Grass => GREEN,
            Block::Dirt => BROWN,
            Block::Stone => GRAY_500,
            Block::BedRock => GRAY,
            Block::Snow => WHITE,
            Block::Sand => SANDY_BROWN,
            Block::Other(val) => return Color::hsl(*val as f32 * (1. / 360.), 1., 0.5),
        }
        .into()
    }
    fn id(&self) -> u32 {
        match self {
            Block::Void => 0,
            Block::Grass => 1,
            Block::Dirt => 2,
            Block::Stone => 3,
            Block::BedRock => 4,
            Block::Snow => 5,
            Block::Sand => 6,
            Block::Other(val) => 0x80000000 | (*val as u32),
        }
    }
    fn from_id(id: u32) -> Self {
        match id {
            0 => Block::Void,
            1 => Block::Grass,
            2 => Block::Dirt,
            3 => Block::Stone,
            4 => Block::BedRock,
            5 => Block::Snow,
            6 => Block::Sand,
            other => {
                if other & 0x80000000 != 0 {
                    Block::Other((other & 0x7FFFFFFF) as u8)
                } else {
                    error!("Invalid block id: {}", other);
                    Block::Void
                }
            }
        }
    }
    fn block_type(&self) -> u8 {
        match self {
            Block::Void => 0,
            Block::Grass => 1,
            Block::Dirt => 2,
            Block::Stone => 3,
            Block::BedRock => 4,
            Block::Snow => 5,
            Block::Sand => 6,
            Block::Other(id) => 128 + (*id >> 1),
        }
    }
}
