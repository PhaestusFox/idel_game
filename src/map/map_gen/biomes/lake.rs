use super::*;

pub struct Lake {
    depth_curve: bevy::math::curve::EasingCurve<f32>,
}

impl Lake {
    pub fn new() -> Self {
        Self {
            depth_curve: bevy::math::curve::EasingCurve::new(
                0.,
                16.,
                bevy::math::curve::easing::EaseFunction::Linear,
            ),
        }
    }
}

impl BiomeDescriptor for Lake {
    fn strength(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> Option<f32> {
        let p = point.as_vec2() * 0.023;
        let rainfall = noise.get([p.x as f64, p.y as f64]) as f32;
        if rainfall > 0.3 {
            Some((1. - (rainfall - 0.3) * 3.).abs())
        } else {
            None
        }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &noise::Fbm<noise::OpenSimplex>,
        ground_level: i32,
    ) -> [Block; CHUNK_SIZE] {
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground_level {
            return data;
        }
        let pos = origin.as_vec3() * PI;
        let d = noise.get([(pos.x * 0.1) as f64, (pos.z * 0.1) as f64]) as f32;
        let wl = self
            .depth_curve
            .sample_unchecked((d * 0.5 + 0.5).clamp(0., 1.)) as i32;
        let r_ground = (ground_level - origin.y).clamp(0, CHUNK_SIZE as i32);
        for y in 0..r_ground as usize {
            let true_y = origin.y + y as i32;
            if true_y == ground_level - 1 {
                data[y] = Block::Grass;
                continue;
            }
            let block = if true_y > ground_level - wl {
                Block::Water
            } else {
                Block::Stone
            };
            data[y] = block;
        }
        data
    }
    fn ground_height(&self, point: IVec2, noise: &noise::Fbm<noise::OpenSimplex>) -> f32 {
        0.
    }
}
