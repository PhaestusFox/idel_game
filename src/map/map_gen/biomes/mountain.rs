use super::*;

#[derive(Reflect)]
pub struct Hills {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    strength_curve: bevy::math::curve::EasingCurve<f32>,
}

impl Hills {
    pub fn new() -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                0.,
                300.,
                bevy::math::curve::easing::EaseFunction::SineInOut,
            ),
            soil_curve: bevy::math::curve::EasingCurve::new(
                0.,
                4.,
                bevy::math::curve::easing::EaseFunction::ExponentialOut,
            ),
            strength_curve: bevy::math::curve::EasingCurve::new(
                0.,
                1.,
                bevy::math::curve::easing::EaseFunction::ExponentialIn,
            ),
        }
    }
}

impl BiomeDescriptor for Hills {
    fn name(&self) -> &str {
        "Hills"
    }
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        let g = (noise.get::<GroundHeight>(point) + 0.1) * 0.9;
        self.strength_curve.sample_unchecked(g.clamp(0., 1.))
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        let g = descriptor.get::<GroundHeight>(point);
        if g > -0.1 && g < 0.4 {
            ((g + 0.1) * 128.0) as u8
        } else {
            0
        }
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
        let t = noise.get::<Fertility>(IVec2::new(origin.x, origin.z));
        let t = (t * 0.5 + 0.5).clamp(0., 1.);
        let soild_depth = self.soil_curve.sample_unchecked(t) as i32;
        for y in 0..r_ground as usize {
            let true_y = origin.y + y as i32;
            let block = if true_y > ground - soild_depth {
                Block::IronOx
            } else {
                Block::Stone
            };
            data[y] = block;
        }
        data
    }
    fn ground_height(&self, point: IVec2, descriptor: &MapDescriptor) -> f32 {
        let ground_l = (descriptor.get::<GroundHeight>(point) + 0.1) * 0.9;
        let g = self.ground_curve.sample_unchecked(ground_l);
        // println!("Ground height: {}", g);
        g
    }
}

#[derive(Reflect)]
pub struct Mountain {
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    frost_line: i32,
    snow_line: i32,
    strength_curve: bevy::math::curve::EasingCurve<f32>,
}

impl Mountain {
    pub fn new() -> Self {
        Self {
            ground_curve: bevy::math::curve::EasingCurve::new(
                0.,
                700.,
                bevy::math::curve::easing::EaseFunction::QuadraticIn,
            ),
            soil_curve: bevy::math::curve::EasingCurve::new(
                0.,
                4.,
                bevy::math::curve::easing::EaseFunction::ExponentialOut,
            ),
            frost_line: 100,
            snow_line: 128,
            strength_curve: bevy::math::curve::EasingCurve::new(
                0.,
                1.,
                bevy::math::curve::easing::EaseFunction::ExponentialIn,
            ),
        }
    }
}

impl BiomeDescriptor for Mountain {
    fn name(&self) -> &str {
        "Mountain"
    }
    fn strength(&self, point: IVec2, noise: &MapDescriptor) -> f32 {
        let g = (noise.get::<GroundHeight>(point) - 0.2) * 2.;
        let s = self.strength_curve.sample_unchecked(g.clamp(0., 1.));
        s
    }
    fn priority(&self, point: IVec2, descriptor: &MapDescriptor) -> u8 {
        let g = descriptor.get::<GroundHeight>(point);
        if g > 0.2 { (g * 255.) as u8 } else { 0 }
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
        // if the top block is in the chunk, set it to the correct block type
        if origin.y > ground - CHUNK_SIZE as i32 {
            let block = if ground > self.snow_line {
                Block::Snow
            } else {
                Block::Grass
            };
            data[r_ground as usize] = Block::Snow;
        }
        let t = noise.get::<Fertility>(IVec2::new(origin.x, origin.z));
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
        let ground_l = descriptor.get::<GroundHeight>(point);
        let g = self.ground_curve.sample_unchecked(ground_l);
        // println!("Ground height: {}({})", g, ground_l);
        g
    }
}
