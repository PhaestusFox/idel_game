use core::f32;

use super::*;
use noise::NoiseFn;
use rand::{RngExt, SeedableRng};
pub struct MapDescriptor {
    noise: noise::Fbm<noise::OpenSimplex>,
    ground_level: i32,
}

impl Default for MapDescriptor {
    fn default() -> Self {
        Self {
            noise: noise::Fbm::new(0),
            ground_level: 16,
        }
    }
}

const CHUNK_SIZE: f32 = super::chunk::CHUNK_SIZE as f32;

impl MapDescriptor {
    pub fn get_block(&self, pos: IVec3) -> u8 {
        if pos == IVec3::ONE {
            println!("pos: {:?}", pos);
        }
        let p = (pos.as_vec3() + f32::consts::PI) * 0.01;
        let v = self.noise.get([p.x as f64, p.y as f64, p.z as f64]) * 5.;
        if pos.y < self.ground_level {
            // if pos.x < 16 && pos.y < 16 && pos.z < 16 && pos.x > 0 && pos.y > 0 && pos.z > 0 {
            //     println!("v: {}", v);
            // }
            if v > 0. {
                (v * 255.).clamp(1., 255.) as u8
            } else {
                0
            }
        } else {
            0
        }
    }
}
