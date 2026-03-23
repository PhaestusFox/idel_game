use core::f32;
use std::borrow::Cow;

use super::*;
use noise::{MultiFractal, NoiseFn};
pub struct MapDescriptor {
    noise: noise::Fbm<noise::OpenSimplex>,
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
            // Box::new(Mountain::new()),
            // Box::new(Plains::new(0.)),
            // Box::new(Desert::new(0.)),
            // Box::new(Ocean::new()),
            Box::new(DebugBiome::new(DebugBiomeType::Height, 3)),
            // Box::new(Plains::new(-0.2)),
            // Box::new(Plains::new(0.2)),
            // Box::new(Desert::new(0.2)),
            // Box::new(Desert::new(-0.2)),
        ]);
        Self {
            noise: noise::Fbm::new(seed),
            biomes,
            #[cfg(feature = "profile")]
            timings,
        }
    }

    pub fn set_octaves(&mut self, octaves: usize) {
        let n = std::mem::take(&mut self.noise);
        self.noise = n.set_octaves(octaves);
    }
    pub fn set_frequency(&mut self, frequency: f64) {
        self.noise.frequency = frequency;
    }
    pub fn set_lacunarity(&mut self, lacunarity: f64) {
        self.noise.lacunarity = lacunarity;
    }
    pub fn set_persistence(&mut self, persistence: f64) {
        let n = std::mem::take(&mut self.noise);
        self.noise = n.set_persistence(persistence);
    }

    pub fn biomes_mut(&mut self) -> &mut Vec<Box<dyn BiomeDescriptor>> {
        self.biomes.get_mut().unwrap()
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

    fn calculate_biome(&self, x: i32, z: i32) -> (usize, i32) {
        let point = IVec2::new(x, z);
        let biomes = self.biomes.read().unwrap();
        let mut top3 = [(0, 0.); 3];
        for (index, biome) in biomes.iter().enumerate() {
            if let Some(strength) = biome.strength(point, self) {
                if strength > top3[0].1 {
                    top3[2] = top3[1];
                    top3[1] = top3[0];
                    top3[0] = (index, strength);
                } else if strength > top3[1].1 {
                    top3[2] = top3[1];
                    top3[1] = (index, strength);
                } else if strength > top3[2].1 {
                    top3[2] = (index, strength);
                }
            }
        }
        let mut ground_level = 0.;
        let tw = top3[0].1 + top3[1].1 + top3[2].1;
        for (biome, weight) in top3 {
            let b = &biomes[biome];
            let w = weight / tw;
            ground_level += w * b.ground_height(point, self);
        }
        (top3[0].0, ground_level as i32)
    }

    #[inline(always)]
    fn get<T: FromMap>(&self, point: T::Point) -> T::Output {
        T::from_map(self, point)
    }
    #[inline(always)]
    fn sample_noise_2d(&self, x: f32, y: f32) -> f64 {
        self.noise.get([x as f64, y as f64])
    }
    #[inline(always)]
    fn sample_noise_3d(&self, x: f32, y: f32, z: f32) -> f64 {
        self.noise.get([x as f64, y as f64, z as f64])
    }
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
