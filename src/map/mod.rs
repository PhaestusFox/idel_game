use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::Task,
};

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
use chunk::*;

use crate::rendering::CustomMaterial;

fn spawn_test_chunk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut chunk_manager: ResMut<MeshGenerator>,
) {
    for x in -16..=16 {
        for y in -2..3 {
            for z in -16..=16 {
                chunk_manager.que(IVec3::new(x, y, z));
            }
        }
    }
}

#[derive(Resource)]
struct MeshGenerator {
    mesh: Handle<Mesh>,
    mesh2: Handle<Mesh>,
    tasks: HashMap<Entity, Task<(ChunkData, Image)>>,
    new_tasks: HashMap<Entity, Task<(ChunkData, Image)>>,
    que: HashSet<IVec3>,
}

impl FromWorld for MeshGenerator {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            mesh2: asset_server.add(make_baked_mesh()),
            mesh: asset_server.add(make_baked_mesh_lod_2()),
            tasks: HashMap::default(),
            new_tasks: HashMap::default(),
            que: HashSet::default(),
        }
    }
}

impl MeshGenerator {
    fn generate(&mut self, commands: &mut Commands, asset_server: Res<AssetServer>) {
        if self.que.is_empty() {
            return;
        }
        let pool = bevy::tasks::AsyncComputeTaskPool::get();
        println!("Generating {} chunks", self.que.len());
        for chunk in self.que.drain() {
            let task = pool.spawn(async move { ChunkData::generate(chunk) });
            let dis = chunk.as_vec3().length();
            let lod = if dis < 128. {
                1.
            } else if dis < 256. {
                2.
            } else if dis < 512. {
                4.
            } else if dis < 1024. {
                8.
            } else {
                16.
            };
            let id = commands
                .spawn((
                    Mesh3d(self.mesh.clone()),
                    MeshMaterial3d(asset_server.add(crate::rendering::CustomMaterial {
                        lod,
                        color_texture: None,
                        alpha_mode: AlphaMode::Opaque,
                    })),
                    Transform::from_translation((chunk * CHUNK_SIZE as i32).as_vec3()),
                ))
                .id();
            self.tasks.insert(id, task);
        }
    }

    fn que(&mut self, pos: IVec3) {
        self.que.insert(pos);
    }
}

fn update_mesh_generator(
    mut mesh_generator: ResMut<MeshGenerator>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    chunks: Query<(&MeshMaterial3d<CustomMaterial>)>,
    mut mashes: ResMut<Assets<CustomMaterial>>,
) {
    let mut keep = HashMap::default();
    for (id, task) in mesh_generator.tasks.drain() {
        if task.is_finished() {
            let Ok(material) = chunks.get(id) else {
                error!("Chunk entity was despawned before mesh generation finished");
                continue;
            };
            let (data, image) = bevy::tasks::block_on(task.cancel()).expect("checked was finished");
            commands.entity(id).insert(Chunk {
                data: asset_server.add(data),
                pos: IVec3::ZERO,
            });
            let Some(mut material) = mashes.get_mut(material.id()) else {
                error!("CustomMaterial asset was removed before mesh generation finished");
                continue;
            };
            material.color_texture = Some(asset_server.add(image));
        } else {
            keep.insert(id, task);
        }
    }
    mesh_generator.tasks = keep;
    mesh_generator.generate(&mut commands, asset_server);
}
