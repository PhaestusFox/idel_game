use super::*;

#[derive(Reflect)]
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
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        0.
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        let rainfall = descriptor.get::<RainFall>(point);
        if rainfall < 0.3 {
            0
        } else {
            (rainfall * 64.) as u8
        }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        noise: &MapDescriptor,
        ground_level: i32,
    ) -> [Block; CHUNK_SIZE] {
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > ground_level {
            return data;
        }
        let d = noise.get::<GroundHeight>(IVec2::new(origin.x, origin.z)) as f32;
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
    fn ground_height(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        0.
    }
}
