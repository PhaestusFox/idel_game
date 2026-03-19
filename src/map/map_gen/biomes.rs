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

use bevy::prelude::*;
use noise::NoiseFn;
use std::f32::consts::PI;

use crate::map::{Block, CHUNK_SIZE};
pub trait BiomeDescriptor: 'static + Send + Sync {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    fn strength(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> Option<f32>;
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &noise::Fbm<noise::OpenSimplex>,
        ground: i32,
    ) -> [Block; CHUNK_SIZE];
    fn ground_height(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> f32;
}
