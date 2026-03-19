use super::*;

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
                bevy::math::curve::easing::EaseFunction::ExponentialInOut,
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
    fn strength(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> Option<f32> {
        let p = point.as_vec2() * 0.023;
        let rainfall = noise.get([p.x as f64, p.y as f64]) as f32;
        if rainfall > self.max_rainfall {
            None
        } else {
            Some(rainfall.abs())
        }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &noise::Fbm<noise::OpenSimplex>,
        ground_level: i32,
    ) -> [Block; CHUNK_SIZE] {
        let pos = origin.as_vec3() * PI;

        let r_ground = (ground_level - origin.y).clamp(0, CHUNK_SIZE as i32);
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground_level {
            return data;
        }
        // if the top block is in the chunk, set it to the correct block type
        if origin.y > ground_level - CHUNK_SIZE as i32 {
            data[r_ground as usize] = Block::Sand;
        }
        let ground_l = (noise.get([(pos.x * 0.01) as f64, (pos.z * 0.01) as f64])) as f32;
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        let soild_depth = self.soil_curve.sample_unchecked(t) as i32;
        for (y, p) in data.iter_mut().enumerate().take(r_ground as usize) {
            let true_y = origin.y + y as i32;
            let block = if true_y > ground_level - soild_depth {
                Block::Sand
            } else {
                Block::Stone
            };
            *p = block;
        }
        data
    }
    fn ground_height(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> f32 {
        let pos = point.as_vec2() * 0.001;
        let ground_l = (noise.get([(pos.x) as f64, (pos.y) as f64])) as f32;
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        self.ground_curve.sample_unchecked(t)
    }
}
