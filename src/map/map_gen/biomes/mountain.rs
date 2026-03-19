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
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> Option<f32> {
        let g = noise.get::<GroundHeight>(point) as f32;
        if g < 0.5 { None } else { Some(g * 2.) }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &MapDescriptor,
        ground: i32,
    ) -> [Block; CHUNK_SIZE] {
        let r_ground = (ground - origin.y).clamp(0, CHUNK_SIZE as i32);
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground {
            return data;
        }
        let t = noise.get::<Fertility>(IVec2::new(origin.x, origin.z)) as f32;
        // if the top block is in the chunk, set it to the correct block type
        if origin.y > ground - CHUNK_SIZE as i32 {
            let block = if ground > self.snow_line {
                Block::Snow
            } else {
                Block::Grass
            };
            data[r_ground as usize] = Block::Snow;
        }
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
    fn ground_height(&self, point: IVec2, descriptor: &MapDescriptor) -> f32 {
        let ground_l = (descriptor.get::<GroundHeight>(point)) as f32;
        self.ground_curve.sample_unchecked(ground_l)
    }
}
