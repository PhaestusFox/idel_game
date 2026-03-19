use super::*;

pub struct DebugBiome {
    scale: u32,
    param: DebugBiomeType,
}

impl DebugBiome {
    pub fn new(param: DebugBiomeType, scale: u32) -> Self {
        Self { param, scale }
    }
}

pub enum DebugBiomeType {
    Height,
}

impl BiomeDescriptor for DebugBiome {
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> Option<f32> {
        Some(1.)
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &MapDescriptor,
        _ground: i32,
    ) -> [Block; CHUNK_SIZE] {
        let mut data = [Block::Void; CHUNK_SIZE];
        let p = origin * self.scale as i32;
        let h = match self.param {
            DebugBiomeType::Height => {
                (noise.get::<GroundHeight>(IVec2::new(p.x, p.z)) * 0.5 + 0.5) * CHUNK_SIZE as f32
            }
        };
        for y in 0..CHUNK_SIZE as usize {
            if h as usize > y {
                data[y] = Block::Other(h as u8 * 4);
            } else {
                data[y] = Block::Void;
            }
        }
        data
    }
    fn ground_height(&self, _point: IVec2, _noise: &MapDescriptor) -> f32 {
        0.
    }
}
