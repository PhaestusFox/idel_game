use super::*;

#[derive(Reflect, Clone)]
pub struct Badland {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
}

impl Badland {
    pub fn new() -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                -16.,
                16.,
                bevy::math::curve::easing::EaseFunction::Linear,
            ),
        }
    }
}

impl BiomeDescriptor for Badland {
    fn name(&self) -> &str {
        "Badland"
    }
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        0.0
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        0
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &MapDescriptor,
        water_table: i32,
    ) -> [Block; CHUNK_SIZE] {
        let ground_l = noise.get::<GroundHeight>(IVec2::new(origin.x, origin.z)) as f32;
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        let ground_level = self.ground_curve.sample_unchecked(t) as i32;
        let r_ground = (ground_level - origin.y).clamp(0, CHUNK_SIZE as i32);
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground_level {
            return data;
        }
        for y in 0..r_ground as usize {
            data[y] = Block::Other(127);
        }
        data
    }
    fn ground_height(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        let ground_l = noise.get::<GroundHeight>(point) as f32;
        self.ground_curve.sample_unchecked(ground_l)
    }
}
