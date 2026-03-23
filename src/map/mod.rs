use std::sync::Arc;
use std::sync::RwLock;

use bevy::ecs::entity;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::{platform::collections::HashMap, prelude::*, tasks::Task};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<chunk::ChunkData>();
        app.init_resource::<ChunkLookup>();
        app.init_resource::<ChunkGenerator>();
        app.add_systems(Startup, spawn_test_chunk);
        app.add_systems(PreUpdate, hide_empty_chunks);
        app.add_systems(PreUpdate, update_mesh_generator);

        app.add_plugins(ambiance::plugin);

        app.init_resource::<MapDescriptor>();

        #[cfg(debug_assertions)]
        app.add_plugins(debug::MapDebugConsolePlugin);
    }
}
pub mod debug;

mod chunk;
mod map_gen;
pub use chunk::*;
use indexmap::IndexSet;
use map_gen::biomes::*;
mod ambiance;

use crate::rendering::CustomMaterial;

const MAP_SIZE_Z: i32 = 16;
const MAP_SIZE_X: i32 = 16;
const MAP_DEPTH: i32 = 0;
fn spawn_test_chunk(mut chunk_manager: ResMut<ChunkGenerator>, map: Res<MapDescriptor>) {
    for x in -map.world_size.x..map.world_size.x {
        for y in -map.world_size.y..=map.world_size.y {
            for z in -map.world_size.z..map.world_size.z {
                chunk_manager.que(IVec3::new(x, y, z));
            }
        }
    }
}

#[derive(Resource)]
pub struct MapDescriptor {
    pub seed: u32,
    pub world_size: IVec3,
}

impl Default for MapDescriptor {
    fn default() -> Self {
        Self {
            seed: 0,
            world_size: IVec3::new(MAP_SIZE_X, MAP_DEPTH, MAP_SIZE_Z),
        }
    }
}

#[derive(Resource)]
pub struct ChunkGenerator {
    main_mesh: Handle<Mesh>,
    meshs: HashMap<LoD, Handle<Mesh>>,
    dummy_image: Handle<Image>,
    tasks: HashMap<ChunkId, Task<ChunkData>>,
    new_tasks: HashMap<ChunkId, Task<ChunkData>>,
    root: Entity,
    dirty: bool,
    max_chunk_tasks: usize,
    que: IndexSet<IVec3>,
    map: Arc<RwLock<map_gen::MapDescriptor>>,
    #[cfg(feature = "profile")]
    timings: (
        std::sync::Mutex<std::sync::mpsc::Receiver<std::time::Duration>>,
        f32,
        usize,
    ),
}

impl FromWorld for ChunkGenerator {
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
        lods.insert(LoD::Solid, asset_server.add(make_solid_mesh()));

        #[cfg(feature = "profile")]
        let (send, rec) = std::sync::mpsc::channel();

        Self {
            root,
            main_mesh: asset_server.add(make_baked_mesh()),
            meshs: lods,
            tasks: HashMap::default(),
            new_tasks: HashMap::default(),
            que: IndexSet::default(),
            dirty: false,
            dummy_image: asset_server.add(ChunkData::dummy_image()),
            max_chunk_tasks: 250,
            #[cfg(not(feature = "profile"))]
            map: Arc::new(RwLock::new(map_gen::MapDescriptor::default())),
            #[cfg(feature = "profile")]
            map: Arc::new(RwLock::new(map_gen::MapDescriptor::new(0, send))),
            #[cfg(feature = "profile")]
            timings: (std::sync::Mutex::new(rec), 0.0, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LoD {
    LOD1,
    LOD2,
    LOD4,
    LOD8,
    LOD16,
    Solid,
    Empty,
}
impl LoD {
    fn step(&self) -> f32 {
        match self {
            LoD::LOD1 => 1.,
            LoD::LOD2 => 2.,
            LoD::LOD4 => 4.,
            LoD::LOD8 => 8.,
            LoD::LOD16 => 16.,
            _ => 1.,
        }
    }
}

impl ChunkGenerator {
    fn generate(
        &mut self,
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        player_pos: Vec3,
        lookup: &ChunkLookup,
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
        let colors = asset_server.load("RtoG.png");
        println!("Generating {} chunks", que.len());
        for chunk in que {
            self.que.swap_remove(&chunk);
            let descriptor = self.map.clone();
            let ass = asset_server.clone();
            let task = pool.spawn(async move {
                let descriptor = descriptor.read().unwrap();
                ChunkData::generate(chunk, &descriptor, ass)
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
            let mesh = self.get_mesh(LoD::LOD1);
            let id = ChunkId::from_ivec3(chunk);
            if lookup.get(&id).is_none() {
                commands.spawn((
                    Name::new(format!("Chunk ({})", chunk)),
                    Mesh3d(mesh),
                    MeshMaterial3d(asset_server.add(crate::rendering::CustomMaterial {
                        lod: lod.step(),
                        color_texture: Some(colors.clone()),
                        alpha_mode: AlphaMode::Opaque,
                        data: self.dummy_image.clone(),
                    })),
                    Transform::from_scale(Vec3::splat(CHUNK_SIZE as f32 * 0.5)),
                    ChildOf(self.root),
                    id,
                ));
            }
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

    pub fn dummy_image(&self) -> Handle<Image> {
        self.dummy_image.clone()
    }
    #[inline(always)]
    pub fn set_octaves(&mut self, octaves: usize) {
        self.map.write().unwrap().set_octaves(octaves);
    }
    #[inline(always)]
    pub fn set_frequency(&mut self, frequency: f64) {
        self.map.write().unwrap().set_frequency(frequency);
    }
    #[inline(always)]
    pub fn set_lacunarity(&mut self, lacunarity: f64) {
        self.map.write().unwrap().set_lacunarity(lacunarity);
    }
    #[inline(always)]
    pub fn set_persistence(&mut self, persistence: f64) {
        self.map.write().unwrap().set_persistence(persistence);
    }
}

fn update_mesh_generator(
    mut mesh_generator: ResMut<ChunkGenerator>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    chunks: Query<&MeshMaterial3d<CustomMaterial>>,
    mut mashes: ResMut<Assets<CustomMaterial>>,
    player: Single<&Transform, With<crate::player::PlayerEntity>>,
    mut done: Local<bool>,
    lookup: Res<ChunkLookup>,
) {
    let mut tasks = std::mem::take(&mut mesh_generator.tasks);
    for (id, task) in tasks.drain() {
        if task.is_finished() {
            let Some(entity) = lookup.get(&id) else {
                error!("Chunk entity was despawned before mesh generation finished");
                continue;
            };
            let Ok(material) = chunks.get(entity) else {
                error!("Chunk entity was despawned before mesh generation finished");
                continue;
            };
            let data = bevy::tasks::block_on(task.cancel()).expect("checked was finished");
            let Some(material) = mashes.get_mut(material.id()) else {
                error!("CustomMaterial asset was removed before mesh generation finished");
                continue;
            };
            if data.images.is_none() {
                error!("ChunkData did not have an image after generation");
                continue;
            }
            let Some(image) = data.images.clone() else {
                error!("ChunkData did not have an image after generation");
                continue;
            };
            material.data = image;
            let chunk = Chunk {
                lod_hint: data.lod_hint(),
                data: asset_server.add(data),
            };
            let mut chunk_entity = commands.entity(entity);
            match chunk.lod_hint {
                // LoD::Solid => chunk_entity.insert(Mesh3d(mesh_generator.get_mesh(LoD::Solid))),
                LoD::Empty => chunk_entity.insert(Visibility::Hidden),
                _ => &mut chunk_entity,
            }
            .insert(chunk);
        } else {
            mesh_generator.new_tasks.insert(id, task);
        }
    }
    std::mem::swap(&mut tasks, &mut mesh_generator.new_tasks);
    mesh_generator.tasks = tasks;
    #[cfg(feature = "profile")]
    {
        let mes_gen = mesh_generator.as_mut();
        for time in mes_gen.timings.0.lock().unwrap().try_iter() {
            mes_gen.timings.1 += time.as_secs_f32();
            mes_gen.timings.2 += 1;
        }
    }
    if mesh_generator.tasks.is_empty() && !*done && mesh_generator.que.is_empty() {
        println!("All chunks generated");
        #[cfg(feature = "profile")]
        {
            println!(
                "Total chunk generation time: {:.2} seconds",
                mesh_generator.timings.1
            );
            let avg = if mesh_generator.timings.2 > 0 {
                mesh_generator.timings.1 / mesh_generator.timings.2 as f32
            } else {
                0.
            };
            println!(
                "Average chunk generation time: {:.2} ms ({} samples)",
                avg * 1000.,
                mesh_generator.timings.2
            );
        }
        *done = true;
    }
    if !mesh_generator.que.is_empty() {
        *done = false;
    }
    if mesh_generator.tasks.len() > mesh_generator.max_chunk_tasks {
        return;
    }
    mesh_generator.generate(&mut commands, asset_server, player.translation, &lookup);
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
    Water = 7,
    Other(u8),
}

fn hide_empty_chunks(mut chunks: Query<(&Chunk, &mut Visibility), Changed<Chunk>>) {
    for (chunk, mut visibility) in &mut chunks {
        if chunk.lod_hint == LoD::Empty {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

impl Block {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Void | Block::Water)
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
            Block::Water => 7,
            Block::Other(val) => *val as u32,
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
            Block::Water => 7,
            Block::Other(id) => *id,
        }
    }
}

#[derive(Resource)]
pub struct ChunkLookup {
    map_root: Entity,
    chunks: HashMap<ChunkId, Entity>,
    blocks: HashMap<ChunkBlock, Entity>,
}

impl FromWorld for ChunkLookup {
    fn from_world(world: &mut World) -> Self {
        Self {
            map_root: world
                .spawn((
                    Name::new("MapRoot"),
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id(),
            chunks: HashMap::default(),
            blocks: HashMap::default(),
        }
    }
}
impl ChunkLookup {
    pub fn insert(&mut self, pos: ChunkId, entity: Entity) {
        self.chunks.insert(pos, entity);
    }

    pub fn remove(&mut self, pos: &ChunkId) {
        self.chunks.remove(pos);
    }

    pub fn get(&self, pos: &ChunkId) -> Option<Entity> {
        self.chunks.get(pos).cloned()
    }

    pub fn add_block(&mut self, pos: ChunkBlock, entity: Entity) {
        self.blocks.insert(pos, entity);
    }

    #[inline]
    pub fn get_block(&self, chunk: &ChunkId) -> Option<Entity> {
        let block = *chunk / CHUNK_BLOCK_SIZE;
        self.blocks.get(&ChunkBlock(*block)).cloned()
    }

    pub fn root(&self) -> Entity {
        self.map_root
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug, Hash, Deref)]
#[component(immutable, on_insert = ChunkBlock::on_insert, on_remove = ChunkBlock::on_remove)]
#[require(Name = Name::new("ChunkBlock"), Transform, Visibility)]
pub struct ChunkBlock(pub IVec3);

const CHUNK_BLOCK_SIZE: i32 = 3;

impl ChunkBlock {
    pub fn world_pos(&self) -> Vec3 {
        let offset = self.0 * CHUNK_BLOCK_SIZE * CHUNK_SIZE as i32;
        (offset).as_vec3()
    }

    pub fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let offset = *world.resource::<ChunkId>();
        let id = *world
            .get::<ChunkBlock>(ctx.entity)
            .expect("ChunkBlock just Inserted");
        let mut lookup = world.resource_mut::<ChunkLookup>();
        lookup.add_block(id, ctx.entity);
        let root = lookup.root();
        world.commands().entity(ctx.entity).insert(ChildOf(root));
        world
            .get_mut::<Name>(ctx.entity)
            .expect("Name is required")
            .set(format!("ChunkBlock: ({},{},{})", id.x, id.y, id.z));

        world
            .get_mut::<Transform>(ctx.entity)
            .expect("Transform is required")
            .translation = id.world_pos() - offset.offset();
    }
    pub fn on_remove(mut world: DeferredWorld, ctx: HookContext) {
        let id = *world
            .get::<ChunkBlock>(ctx.entity)
            .expect("ChunkBlock about to be Removed");
        world.resource_mut::<ChunkLookup>().blocks.remove(&id);
    }
}

impl From<ChunkId> for ChunkBlock {
    fn from(value: ChunkId) -> Self {
        Self(*value / CHUNK_BLOCK_SIZE)
    }
}

impl std::ops::Sub<ChunkBlock> for ChunkId {
    type Output = ChunkId;

    fn sub(self, rhs: ChunkBlock) -> Self::Output {
        let s = *rhs * CHUNK_BLOCK_SIZE;
        ChunkId::from_ivec3(*self - s)
    }
}

#[derive(SystemParam)]
pub struct Map<'w> {
    lookup: Res<'w, ChunkLookup>,
    data: Res<'w, Assets<ChunkData>>,
    world_offset: Res<'w, ChunkId>,
}

impl<'w> Map<'w> {
    pub fn get_block(&self, pos: Vec3, chunks: Query<&Chunk>) -> Result<Block, MapError> {
        let pos = pos.floor() + Vec3::splat(CHUNK_SIZE as f32 * 0.5);
        let chunk_id = ChunkId::from_translation(pos) + *self.world_offset;
        let Some(chunk_entity) = self.lookup.get(&chunk_id) else {
            return Err(MapError::NoEntity);
        };
        let Ok(chunk) = chunks.get(chunk_entity) else {
            return Err(MapError::NoChunk);
        };
        let Some(chunk_data) = self.data.get(&chunk.data) else {
            return Err(MapError::NoChunkData);
        };
        let foot = (pos).as_ivec3().rem_euclid(IVec3::splat(CHUNK_SIZE as i32));
        let block = chunk_data.get_block(foot.x as u8, foot.y as u8, foot.z as u8);
        Ok(block)
    }
}

pub enum MapError {
    NoEntity,
    NoChunk,
    NoChunkData,
    NoBlock,
}
