use core::f32;

use super::*;
use noise::NoiseFn;
use rand::{RngExt, SeedableRng};
pub struct MapDescriptor {
    noise: noise::Fbm<noise::OpenSimplex>,
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    frost_line: i32,
    snow_line: i32,
    ground_level: f32,
}

impl Default for MapDescriptor {
    fn default() -> Self {
        Self {
            noise: noise::Fbm::new(0),
            ground_curve: bevy::math::curve::EasingCurve::new(
                -64.,
                700.,
                bevy::math::curve::easing::EaseFunction::CubicIn,
            ),
            soil_curve: bevy::math::curve::EasingCurve::new(
                0.,
                16.,
                bevy::math::curve::easing::EaseFunction::ExponentialOut,
            ),
            ground_level: 32.0,
            frost_line: 100,
            snow_line: 128,
        }
    }
}

const CHUNK_SIZE: f32 = super::chunk::CHUNK_SIZE as f32;

use bevy::math::Curve;
impl MapDescriptor {
    pub fn get_block(&self, pos: IVec3) -> Block {
        let offset = pos.as_vec3() * f32::consts::PI;
        let p = offset * 0.006;
        let water_l = (self.noise.get([p.x as f64, p.z as f64])) as f32;
        let water_level = (water_l * self.ground_level).floor() as i32;

        let p = offset * 0.001;
        let ground_l = ((self
            .noise
            .get([p.x as f64, p.z as f64, water_l as f64 * 0.001])) as f32);
        let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
        let ground_level = self.ground_curve.sample_unchecked(t) as i32;

        let p = offset * 0.01;
        let v = self.noise.get([p.x as f64, p.y as f64, p.z as f64]);
        // above ground is air
        if pos.y > ground_level {
            return Block::Void;
        }
        // above snow line ground is snow
        if pos.y > self.snow_line && pos.y == ground_level {
            return Block::Snow;
        }
        // above frost there is no soil, only stone
        if pos.y > self.frost_line {
            return Block::Stone;
        }

        // below water level is bedrock
        if pos.y < water_level {
            return Block::BedRock;
        }
        let soil_level = self.soil_curve.sample_unchecked(t) as i32;
        // the top layer is grass
        if pos.y == ground_level {
            return Block::Grass;
        }
        // if we are in the soil layer, return dirt
        if pos.y > ground_level - soil_level {
            return Block::Dirt;
        }
        // otherwise return stone
        Block::Stone
    }
}
