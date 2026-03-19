mod mountain;
pub use mountain::Mountain;
mod plains;
pub use plains::Plains;
mod badlands;
pub use badlands::Badland;

mod desert;
pub use desert::Desert;

mod lake;
pub use lake::Lake;

mod ocean;
pub use ocean::Ocean;

mod debug;
pub use debug::{DebugBiome, DebugBiomeType};

use super::MapDescriptor;
use crate::map::map_gen::map_parameters::*;
use bevy::prelude::*;

use crate::map::{Block, CHUNK_SIZE};
pub trait BiomeDescriptor: 'static + Send + Sync {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    fn strength(&self, point: IVec2, descriptor: &MapDescriptor) -> Option<f32>;
    fn generate_column(
        &self,
        origin: IVec3,
        descriptor: &MapDescriptor,
        ground: i32,
    ) -> [Block; CHUNK_SIZE];
    fn ground_height(&self, point: IVec2, descriptor: &MapDescriptor) -> f32;
}
