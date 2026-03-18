use core::f32;

use super::*;
use noise::NoiseFn;
pub struct MapDescriptor {
    noise: noise::Fbm<noise::OpenSimplex>,
    ground_curve: bevy::math::curve::EasingCurve<f32>,
    soil_curve: bevy::math::curve::EasingCurve<f32>,
    frost_line: i32,
    snow_line: i32,
    ground_level: f32,
    #[cfg(feature = "profile")]
    timings: std::sync::mpsc::Sender<std::time::Duration>,
}

#[cfg(not(feature = "profile"))]
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

impl MapDescriptor {
    pub fn new(
        seed: u32,
        #[cfg(feature = "profile")] timings: std::sync::mpsc::Sender<std::time::Duration>,
    ) -> Self {
        Self {
            noise: noise::Fbm::new(seed),
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
            #[cfg(feature = "profile")]
            timings,
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

    pub fn generate_chunk(&self, chunk_id: ChunkId) -> ChunkData {
        #[cfg(feature = "profile")]
        let start = std::time::Instant::now();
        let mut data = ChunkData::empty();
        let offset = *chunk_id * CHUNK_SIZE as i32;
        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let pos = (offset + IVec3::new(x, 0, z)).as_vec3() * f32::consts::PI;
                let water_l = (self
                    .noise
                    .get([(pos.x * 0.006) as f64, (pos.z * 0.006) as f64]))
                    as f32;
                let water_level = (water_l * self.ground_level).floor() as i32;
                let ground_l = (self.noise.get([
                    (pos.x * 0.001) as f64,
                    (pos.z * 0.001) as f64,
                    water_l as f64 * 0.001,
                ])) as f32;
                let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
                let ground_level = self.ground_curve.sample_unchecked(t) as i32;
                let r_ground = (ground_level - offset.y).clamp(0, CHUNK_SIZE as i32);
                if offset.y > ground_level {
                    continue;
                }
                // if the top block is in the chunk, set it to the correct block type
                if offset.y > ground_level - CHUNK_SIZE as i32 {
                    let block = if ground_level > self.snow_line {
                        Block::Snow
                    } else {
                        Block::Grass
                    };
                    data.set_block(x as u8, r_ground as u8, z as u8, block);
                }
                let soild_depth = self.soil_curve.sample_unchecked(t) as i32;
                for y in 0..r_ground as u8 {
                    let true_y = offset.y + y as i32;
                    let block = if true_y < water_level {
                        Block::BedRock
                    } else if true_y > self.frost_line {
                        Block::Stone
                    } else if true_y > ground_level - soild_depth {
                        Block::Dirt
                    } else {
                        Block::Stone
                    };
                    data.set_block(x as u8, y, z as u8, block);
                }
            }
        }

        #[cfg(feature = "profile")]
        {
            let duration = start.elapsed();
            let _ = self.timings.send(duration);
        }

        data
    }
}
