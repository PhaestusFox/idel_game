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
    for x in 0..=16 {
        for y in 0..2 {
            for z in 0..=8 {
                if z == 1 && x == 1 {
                    continue;
                }
                chunk_manager.que(IVec3::new(x, y, z));
            }
        }
    }
}

#[derive(Resource)]
struct MeshGenerator {
    main_mesh: Handle<Mesh>,
    meshs: Vec<Handle<Mesh>>,
    tasks: HashMap<Entity, Task<(ChunkData, Image)>>,
    new_tasks: HashMap<Entity, Task<(ChunkData, Image)>>,
    root: Entity,
    dirty: bool,
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
        let mut lods = Vec::new();
        // for lod in [LoD::LOD1, LoD::LOD2, LoD::LOD4, LoD::LOD8, LoD::LOD16] {
        //     lods.push(asset_server.add(make_baked_mesh_lod(lod)));
        // }

        Self {
            root,
            main_mesh: asset_server.add(make_baked_mesh()),
            meshs: lods,
            tasks: HashMap::default(),
            new_tasks: HashMap::default(),
            que: IndexSet::default(),
            dirty: false,
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
            } else if dis < 64. {
                LoD::LOD4
            } else if dis < 128. {
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
                        color_texture: None,
                        alpha_mode: AlphaMode::Opaque,
                        chunk_offset: Vec3::ZERO,
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
        self.meshs
            .get(lod as usize)
            .unwrap_or(&self.main_mesh)
            .clone()
    }
}

fn update_mesh_generator(
    mut mesh_generator: ResMut<MeshGenerator>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    chunks: Query<(&ChunkId, &MeshMaterial3d<CustomMaterial>)>,
    mut mashes: ResMut<Assets<CustomMaterial>>,
    player: Single<&Transform, With<crate::player::PlayerEntity>>,
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
                data: asset_server.add(data),
            };
            let Some(material) = mashes.get_mut(material.id()) else {
                error!("CustomMaterial asset was removed before mesh generation finished");
                continue;
            };
            material.color_texture = Some(asset_server.add(image));
            material.chunk_offset = chunk_id.offset();
            commands.entity(id).insert(chunk);
        } else {
            keep.insert(id, task);
        }
    }
    mesh_generator.tasks = keep;
    mesh_generator.generate(&mut commands, asset_server, player.translation);
}
