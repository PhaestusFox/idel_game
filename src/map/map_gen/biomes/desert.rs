use super::*;

#[derive(Reflect)]
pub struct Desert {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    max_rainfall: f32,
}

impl Desert {
    pub fn new(max_rainfall: f32) -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                -16.,
                32.,
                bevy::math::curve::easing::EaseFunction::Linear,
            ),
            soil_curve: bevy::math::curve::EasingCurve::new(
                0.,
                16.,
                bevy::math::curve::easing::EaseFunction::ExponentialOut,
            ),
            max_rainfall,
        }
    }
}

impl BiomeDescriptor for Desert {
    fn name(&self) -> &str {
        "Desert"
    }
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        let rainfall = noise.get::<Fertility>(point);
        rainfall
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        let rainfall = descriptor.get::<Fertility>(point);
        let r = descriptor.get::<RainFall>(point);
        if r > self.max_rainfall {
            0
        } else {
            64 - (rainfall * 64.) as u8
        }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &MapDescriptor,
        ground_level: i32,
    ) -> [Block; CHUNK_SIZE] {
        let r_ground = (ground_level - origin.y).clamp(0, CHUNK_SIZE as i32);
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground_level {
            return data;
        }
        let sand_s = noise.get::<GroundHeight2>(IVec2::new(origin.x, origin.z));
        let sand_depth = self.soil_curve.sample_unchecked(sand_s) as i32;
        for (y, p) in data.iter_mut().enumerate().take(r_ground as usize) {
            let true_y = origin.y + y as i32;
            let block = if true_y >= ground_level - sand_depth {
                Block::Sand
            } else {
                Block::Stone
            };
            *p = block;
        }
        data
    }
    fn ground_height(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        let ground_l = noise.get::<GroundHeight>(point);
        self.ground_curve.sample_unchecked(ground_l)
    }
}
