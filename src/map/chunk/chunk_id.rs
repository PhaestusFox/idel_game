pub use super::*;

#[derive(Component, Deref, DerefMut, Debug, Clone, Copy, PartialEq, Eq, Hash, Resource)]
#[component(on_insert = Self::on_insert, on_remove = Self::on_remove)]
#[require(Transform)]
pub struct ChunkId(IVec3);

impl ChunkId {
    pub fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        // Todo remove this. the player is being ignored so they dont eat the map
        if world.get::<PlayerEntity>(ctx.entity).is_some() {
            return;
        }
        let id = *world.get::<ChunkId>(ctx.entity).unwrap();
        let block = ChunkBlock::from(id);
        let index = id - block;
        world
            .get_mut::<Transform>(ctx.entity)
            .expect("Transform is required")
            .translation = index.offset();

        let lookup = world.resource::<ChunkLookup>();
        let block = if let Some(block_entity) = lookup.get_block(&id) {
            block_entity
        } else {
            world.commands().spawn(block).id()
        };
        world.commands().entity(ctx.entity).insert(ChildOf(block));
        let mut lookup = world.resource_mut::<ChunkLookup>();
        lookup.insert(id, ctx.entity);
    }
    pub fn on_remove(mut world: DeferredWorld, ctx: HookContext) {
        let id = *world.get::<ChunkId>(ctx.entity).unwrap();
        world.resource_mut::<ChunkLookup>().remove(&id);
    }
}

impl Div<i32> for ChunkId {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Default for ChunkId {
    fn default() -> Self {
        Self::ZERO
    }
}

impl ChunkId {
    pub const ZERO: Self = Self(IVec3::ZERO);

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn from_ivec3(vec: IVec3) -> Self {
        Self(vec)
    }

    pub fn from_translation(translation: Vec3) -> Self {
        let pos = (translation / CHUNK_SIZE as f32).floor().as_ivec3();
        Self(pos)
    }

    pub fn offset(&self) -> Vec3 {
        (self.0 * CHUNK_SIZE as i32).as_vec3()
    }
}

impl Add for ChunkId {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for ChunkId {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for ChunkId {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add<IVec3> for ChunkId {
    type Output = Self;

    fn add(self, rhs: IVec3) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<IVec3> for ChunkId {
    fn add_assign(&mut self, rhs: IVec3) {
        self.0 += rhs;
    }
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk({},{},{})", self.0.x, self.0.y, self.0.z)
    }
}
