use core::f32;
use std::{borrow::Cow, num::NonZero};

use super::*;
use noise::{MultiFractal, NoiseFn};
pub struct MapDescriptor {
    h_noise: noise::Fbm<noise::OpenSimplex>,
    m_noise: noise::Fbm<noise::OpenSimplex>,
    l_noise: noise::Fbm<noise::OpenSimplex>,
    biomes: RwLock<Vec<Box<dyn BiomeDescriptor>>>,
    #[cfg(feature = "profile")]
    timings: std::sync::mpsc::Sender<std::time::Duration>,
}

#[cfg(not(feature = "profile"))]
impl Default for MapDescriptor {
    fn default() -> Self {
        Self::new(0)
    }
}

impl MapDescriptor {
    pub fn new(
        seed: u32,
        #[cfg(feature = "profile")] timings: std::sync::mpsc::Sender<std::time::Duration>,
    ) -> Self {
        let biomes: RwLock<Vec<Box<dyn BiomeDescriptor>>> = RwLock::new(vec![
            // Box::new(Badland::new()),
            // Box::new(Hills::new(-0.1)),
            // Box::new(Hills::new(0.0)),
            // Box::new(Hills::new(0.1)),
            // Box::new(Mountain::new(0.3)),
            Box::new(Mountain::new(0.1)),
            // Box::new(Mountain::new(0.2)),
            // Box::new(Plains::new(0.1)),
            // Box::new(Desert::new(0.2)),
            // Box::new(Ocean::new()),
            // Box::new(DebugBiome::new(DebugBiomeType::Height, 1)),
            // Box::new(Plains::new(-0.2)),
            // Box::new(Plains::new(0.2)),
            // Box::new(Desert::new(0.2)),
            // Box::new(Desert::new(-0.2)),
        ]);
        let mut h_noise = noise::Fbm::new(seed);
        let mut m_noise = noise::Fbm::new(seed + 0x5068);
        let mut l_noise = noise::Fbm::new(seed + 0x6f78);
        h_noise.lacunarity = 3f64.sqrt();
        m_noise.lacunarity = 5f64.sqrt();
        l_noise.lacunarity = 11f64.sqrt();
        h_noise.persistence = 0.85;
        m_noise.persistence = 0.5;
        l_noise.persistence = 0.2;
        let h_noise = h_noise.set_octaves(8);
        let m_noise = m_noise.set_octaves(6);
        let l_noise = l_noise.set_octaves(4);
        Self {
            h_noise,
            m_noise,
            l_noise,
            biomes,
            #[cfg(feature = "profile")]
            timings,
        }
    }

    pub fn set_octaves(&mut self, octaves: usize) {
        let n = std::mem::take(&mut self.h_noise);
        self.h_noise = n.set_octaves(octaves);
    }
    pub fn set_frequency(&mut self, frequency: f64) {
        self.h_noise.frequency = frequency;
    }
    pub fn set_lacunarity(&mut self, lacunarity: f64) {
        self.h_noise.lacunarity = lacunarity;
    }
    pub fn set_persistence(&mut self, persistence: f64) {
        let n = std::mem::take(&mut self.h_noise);
        self.h_noise = n.set_persistence(persistence);
    }

    pub fn biomes_mut(&mut self) -> &mut Vec<Box<dyn BiomeDescriptor>> {
        self.biomes.get_mut().unwrap()
    }

    pub fn biomes(&self) -> std::sync::RwLockReadGuard<'_, Vec<Box<dyn BiomeDescriptor>>> {
        self.biomes.read().unwrap()
    }
}

const CHUNK_SIZE: f32 = super::chunk::CHUNK_SIZE as f32;

impl MapDescriptor {
    // pub fn get_block(&self, pos: IVec3) -> Block {
    //     let offset = pos.as_vec3() * f32::consts::PI;
    //     let p = offset * 0.006;
    //     let water_l = (self.noise.get([p.x as f64, p.z as f64])) as f32;
    //     let water_level = (water_l * self.ground_level).floor() as i32;

    //     let p = offset * 0.001;
    //     let ground_l = ((self
    //         .noise
    //         .get([p.x as f64, p.z as f64, water_l as f64 * 0.001])) as f32);
    //     let t = (ground_l * 0.5 + 0.5).clamp(0., 1.);
    //     let ground_level = self.ground_curve.sample_unchecked(t) as i32;

    //     let p = offset * 0.01;
    //     let v = self.noise.get([p.x as f64, p.y as f64, p.z as f64]);
    //     // above ground is air
    //     if pos.y > ground_level {
    //         return Block::Void;
    //     }
    //     // above snow line ground is snow
    //     if pos.y > self.snow_line && pos.y == ground_level {
    //         return Block::Snow;
    //     }
    //     // above frost there is no soil, only stone
    //     if pos.y > self.frost_line {
    //         return Block::Stone;
    //     }

    //     // below water level is bedrock
    //     if pos.y < water_level {
    //         return Block::BedRock;
    //     }
    //     let soil_level = self.soil_curve.sample_unchecked(t) as i32;
    //     // the top layer is grass
    //     if pos.y == ground_level {
    //         return Block::Grass;
    //     }
    //     // if we are in the soil layer, return dirt
    //     if pos.y > ground_level - soil_level {
    //         return Block::Dirt;
    //     }
    //     // otherwise return stone
    //     Block::Stone
    // }

    pub fn generate_chunk(&self, chunk_id: ChunkId) -> ChunkData {
        #[cfg(feature = "profile")]
        let start = std::time::Instant::now();
        let mut data = ChunkData::empty();
        let offset = *chunk_id * CHUNK_SIZE as i32;
        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let (b, ground_level) = self.calculate_biome(offset.x + x, offset.z + z);
                let biomes = self.biomes.read().unwrap();
                let biome = &biomes[b];
                let origin = IVec3::new(x, 0, z) + offset;
                for (y, block) in biome
                    .generate_column(origin, self, ground_level)
                    .into_iter()
                    .enumerate()
                {
                    data.set_block(x as u8, y as u8, z as u8, block);
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

    pub fn calculate_biome(&self, x: i32, z: i32) -> (usize, i32) {
        let top3 = self.calculate_biomes(x, z);
        let biomes = self.biomes.read().unwrap();
        let mut out = [(0., 0.); 3];
        for (i, (index, _)) in top3.into_iter().enumerate() {
            let biome = &biomes[index];
            let strength = biome.strength(IVec2::new(x, z), self);
            out[i] = (strength, biome.ground_height(IVec2::new(x, z), self));
        }
        let total_strength = vec3(out[0].0, out[1].0, out[2].0).normalize();
        let correct = total_strength.x + total_strength.y + total_strength.z;
        let total_strength = total_strength / correct;
        let ground_level = (out[0].1 * total_strength.x
            + out[1].1 * total_strength.y
            + out[2].1 * total_strength.z) as i32;

        (top3[0].0, ground_level as i32)
    }

    pub fn calculate_biomes(&self, x: i32, z: i32) -> [(usize, u8); 3] {
        let point = IVec2::new(x, z);
        let biomes = self.biomes.read().unwrap();
        let mut top3 = [(0, 0); 3];
        for (index, biome) in biomes.iter().enumerate() {
            let p = biome.priority(point, self);
            if p != 0 {
                if p > top3[0].1 {
                    top3[2] = top3[1];
                    top3[1] = top3[0];
                    top3[0] = (index, p);
                } else if p > top3[1].1 {
                    top3[2] = top3[1];
                    top3[1] = (index, p);
                } else if p > top3[2].1 {
                    top3[2] = (index, p);
                }
            }
        }
        top3
    }

    #[inline(always)]
    fn get<T: FromMap>(&self, point: T::Point) -> T::Output {
        T::from_map(self, point)
    }
    #[inline(always)]
    fn sample_noise_2d(&self, x: f32, y: f32, level: Quality) -> f64 {
        match level {
            Quality::Low => self.l_noise.get([x as f64, y as f64]),
            Quality::Medium => self.m_noise.get([x as f64, y as f64]),
            Quality::High => self.h_noise.get([x as f64, y as f64]),
        }
    }
    #[inline(always)]
    fn sample_noise_3d(&self, x: f32, y: f32, z: f32, level: Quality) -> f64 {
        match level {
            Quality::Low => self.l_noise.get([x as f64, y as f64, z as f64]),
            Quality::Medium => self.m_noise.get([x as f64, y as f64, z as f64]),
            Quality::High => self.h_noise.get([x as f64, y as f64, z as f64]),
        }
    }
}

pub enum Quality {
    Low,
    Medium,
    High,
}

trait FromMap: Sized {
    type Point;
    type Output;
    fn from_map(descriptor: &MapDescriptor, point: Self::Point) -> Self::Output;
}

pub mod biomes;
use biomes::*;

mod map_parameters;
use map_parameters::*;
