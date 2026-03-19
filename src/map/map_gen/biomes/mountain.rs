use super::*;
pub struct Mountain {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    frost_line: i32,
    snow_line: i32,
}

impl Mountain {
    pub fn new() -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                -64.,
                700.,
                bevy::math::curve::easing::EaseFunction::CubicIn,
            ),
            soil_curve: bevy::math::curve::EasingCurve::new(
                0.,
                4.,
                bevy::math::curve::easing::EaseFunction::ExponentialOut,
            ),
            frost_line: 100,
            snow_line: 128,
        }
    }
}

impl BiomeDescriptor for Mountain {
    fn name(&self) -> &str {
        "Mountain"
    }
    fn strength(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> Option<f32> {
        let p = point.as_vec2() * 0.001;
        let g = noise.get([p.x as f64, p.y as f64]) as f32;
        if g < 0. { None } else { Some(g * 5.) }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &noise::Fbm<noise::OpenSimplex>,
        ground: i32,
    ) -> [Block; CHUNK_SIZE] {
        let pos = origin.as_vec3() * PI;
        let r_ground = (ground - origin.y).clamp(0, CHUNK_SIZE as i32);
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground {
            return data;
        }
        // if the top block is in the chunk, set it to the correct block type
        if origin.y > ground - CHUNK_SIZE as i32 {
            let block = if ground > self.snow_line {
                Block::Snow
            } else {
                Block::Grass
            };
            data[r_ground as usize] = Block::Snow;
        }
        let t = noise.get([(pos.x * 0.01) as f64, (pos.z * 0.01) as f64]) as f32;
        let t = (t * 0.5 + 0.5).clamp(0., 1.);
        let soild_depth = self.soil_curve.sample_unchecked(t) as i32;
        for y in 0..r_ground as usize {
            let true_y = origin.y + y as i32;
            let block = if true_y > self.frost_line {
                Block::Stone
            } else if true_y > ground - soild_depth {
                Block::Dirt
            } else {
                Block::Stone
            };
            data[y] = block;
        }
        data
    }
    fn ground_height(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> f32 {
        let pos = point.as_vec2() * 0.001;
        let ground_l = (noise.get([pos.x as f64, pos.y as f64])) as f32;
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        self.ground_curve.sample_unchecked(t)
    }
}
