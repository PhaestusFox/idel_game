use super::*;

#[derive(Reflect, Clone)]
pub struct Ocean {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
}

impl Ocean {
    pub fn new() -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                0.,
                -300.,
                bevy::math::curve::easing::EaseFunction::ExponentialInOut,
            ),
        }
    }
}

impl BiomeDescriptor for Ocean {
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        let g = noise.get::<GroundHeight>(point);
        -g * 2.
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        let g = descriptor.get::<GroundHeight>(point);
        if g < -0.3 { (-g * 128.) as u8 } else { 0 }
    }
    fn generate_column(
        &self,
        origin: IVec3,
        descriptor: &MapDescriptor,
        ground_level: i32,
    ) -> [Block; CHUNK_SIZE] {
        let mut data = [Block::Void; CHUNK_SIZE];
        if origin.y > 0 {
            return data;
        }
        for y in 0..CHUNK_SIZE as usize {
            if origin.y + y as i32 > ground_level {
                data[y] = Block::Water;
            } else {
                data[y] = Block::Stone;
            }
        }
        data
    }
    fn ground_height(&self, point: IVec2, descriptor: &MapDescriptor) -> f32 {
        let ground_l = (descriptor.get::<GroundHeight>(point)) as f32;
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        -self.ground_curve.sample_unchecked(t)
    }
}
