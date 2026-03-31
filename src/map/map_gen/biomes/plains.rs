use super::*;

#[derive(Reflect, Clone)]
pub struct Plains {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    min_rainfall: f32,
    strength_curve: bevy::math::curve::EasingCurve<f32>,
}

impl Plains {
    pub fn new(min_rainfall: f32) -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                -8.,
                16.,
                bevy::math::curve::easing::EaseFunction::Linear,
            ),
            soil_curve: bevy::math::curve::EasingCurve::new(
                0.,
                16.,
                bevy::math::curve::easing::EaseFunction::ExponentialOut,
            ),
            min_rainfall,
            strength_curve: bevy::math::curve::EasingCurve::new(
                0.,
                1.,
                bevy::math::curve::easing::EaseFunction::Linear,
            ),
        }
    }
}

impl BiomeDescriptor for Plains {
    fn name(&self) -> &str {
        "Plains"
    }
    fn strength(&self, point: IVec2, descriptor: &MapDescriptor) -> f32 {
        let rainfall = descriptor.get::<RainShadow>(point);
        let t = 1. - (rainfall - self.min_rainfall).abs();
        self.strength_curve.sample_unchecked(t.clamp(0., 1.))
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        let rainfall = descriptor.get::<RainFall>(point);
        if rainfall < self.min_rainfall {
            0
        } else {
            (rainfall * 64.) as u8
        }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        descriptor: &MapDescriptor,
        ground_level: i32,
    ) -> [Block; CHUNK_SIZE] {
        let r_ground = (ground_level - origin.y).clamp(0, CHUNK_SIZE as i32);
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground_level {
            return data;
        }
        let fertility = descriptor.get::<Fertility>(IVec2::new(origin.x, origin.z));
        // if the top block is in the chunk, set it to the correct block type
        if origin.y > ground_level - CHUNK_SIZE as i32 && fertility > 0.3 {
            data[r_ground as usize] = Block::Grass;
        }
        let soild_depth = self.soil_curve.sample_unchecked(fertility) as i32;
        for (y, p) in data.iter_mut().enumerate().take(r_ground as usize) {
            let true_y = origin.y + y as i32;
            let block = if true_y > ground_level - soild_depth {
                Block::Dirt
            } else {
                Block::Stone
            };
            *p = block;
        }
        data
    }
    fn ground_height(&self, point: IVec2, descriptor: &MapDescriptor) -> f32 {
        let ground_l = descriptor.get::<GroundHeight>(point);
        self.ground_curve.sample_unchecked(ground_l)
    }
}
