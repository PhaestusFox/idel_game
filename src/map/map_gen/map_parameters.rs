use std::f32::consts::PI;

use super::{FromMap, MapDescriptor};
use bevy::prelude::*;

pub struct RainFall;

impl FromMap for RainFall {
    type Point = IVec2;
    type Output = f32;
    /// Get the rainfall at a given point. The value is between 0 and 1, where 0 is no rainfall and 1 is maximum rainfall.
    fn from_map(map: &MapDescriptor, point: Self::Point) -> Self::Output {
        let p = point.as_vec2() * 0.023;
        map.sample_noise_2d(p.x, p.y) as f32 * 0.5 + 0.5
    }
}

pub struct GroundHeight;

impl FromMap for GroundHeight {
    type Point = IVec2;
    type Output = f32;
    /// Get the ground height at a given point.<br>
    /// - The value is between -1..1<br>
    /// - -1 is as low as possible<br>
    /// - 1 is as high as possible<br>
    /// - the actual ground hight is decided by the biome<br> This is effecivly how out of nomral the ground height is at this point, and the biome will decide how to interpret this value
    fn from_map(map: &MapDescriptor, point: Self::Point) -> Self::Output {
        let p = point.as_vec2() * 0.001 * PI;
        map.sample_noise_2d(p.x, p.y) as f32
    }
}

pub struct GroundHeight2;

impl FromMap for GroundHeight2 {
    type Point = IVec2;
    type Output = f32;
    /// Get the ground height at a given point.<br>
    /// - The value is between -1..1<br>
    /// - -1 is as low as possible<br>
    /// - 1 is as high as possible<br>
    /// - the actual ground hight is decided by the biome<br> This is effecivly how out of nomral the ground height is at this point, and the biome will decide how to interpret this value
    fn from_map(map: &MapDescriptor, point: Self::Point) -> Self::Output {
        let p = point.as_vec2() * 0.0001 * PI;
        map.sample_noise_2d(p.x, p.y) as f32
    }
}

pub struct Fertility;

impl FromMap for Fertility {
    type Point = IVec2;
    type Output = f32;
    /// Get the fertility at a given point. The value is between 0 and 1, where 0 is no fertility and 1 is maximum fertility.
    fn from_map(map: &MapDescriptor, point: Self::Point) -> Self::Output {
        let p = point.as_vec2() * 0.023;
        map.sample_noise_3d(p.x, p.y, 0.1) as f32 * 0.5 + 0.5
    }
}
