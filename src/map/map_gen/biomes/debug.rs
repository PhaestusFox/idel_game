use super::*;

#[derive(Reflect, Clone)]
pub struct DebugBiome {
    pub scale: u32,
    pub param: DebugBiomeType,
}

impl DebugBiome {
    pub fn new(param: DebugBiomeType, scale: u32) -> Self {
        Self { param, scale }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub enum DebugBiomeType {
    Height,
    Rainfall,
    Fertility,
    GroundHeight2,
    GroundHeight3,
    RainShadow,
}

impl BiomeDescriptor for DebugBiome {
    fn name(&self) -> &str {
        "Debug"
    }
    fn strength(&self, _: IVec2, _: &MapDescriptor) -> f32 {
        1.
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        u8::MAX
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
            DebugBiomeType::Rainfall => {
                noise.get::<RainFall>(IVec2::new(p.x, p.z)) * CHUNK_SIZE as f32
            }
            DebugBiomeType::Fertility => {
                noise.get::<Fertility>(IVec2::new(p.x, p.z)) * CHUNK_SIZE as f32
            }
            DebugBiomeType::GroundHeight2 => {
                (noise.get::<GroundHeight2>(IVec2::new(p.x, p.z)) * 0.5 + 0.5) * CHUNK_SIZE as f32
            }
            DebugBiomeType::GroundHeight3 => {
                (noise.get::<GroundHeight3>(IVec2::new(p.x, p.z)) * 0.5 + 0.5) * CHUNK_SIZE as f32
            }
            DebugBiomeType::RainShadow => {
                noise.get::<RainShadow>(IVec2::new(p.x, p.z)) * CHUNK_SIZE as f32
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
