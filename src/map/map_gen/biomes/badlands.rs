use super::*;

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
    fn strength(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> Option<f32> {
        None
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &noise::Fbm<noise::OpenSimplex>,
        water_table: i32,
    ) -> [Block; CHUNK_SIZE] {
        let pos = origin.as_vec3() * PI;
        let ground_l = (noise.get([
            (pos.x * 0.001) as f64,
            (pos.z * 0.001) as f64,
            water_table as f64 * 0.001,
        ])) as f32;
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
    fn ground_height(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> f32 {
        let pos = point.as_vec2() * 0.001;
        let ground_l = (noise.get([(pos.x * 0.001) as f64, (pos.y * 0.001) as f64])) as f32;
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        self.ground_curve.sample_unchecked(t)
    }
}
