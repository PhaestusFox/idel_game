//! Voxel raycast implementation using DDA (Digital Differential Analyzer)
//!
//! This module provides efficient ray-tracing through a voxel grid by stepping through
//! each block the ray intersects. It uses a modified DDA algorithm adapted for voxel traversal:
//!
//! # Algorithm Overview
//! 1. Start at ray origin with a given direction (forward from camera)
//! 2. For each axis (X, Y, Z), compute:
//!    - `step`: direction sign (+1 or -1)
//!    - `t_delta`: parameter distance to cross one voxel boundary
//!    - `t_max`: parameter distance to next boundary crossing
//! 3. Iteratively step through voxels by finding which axis hits its boundary next
//! 4. For each voxel, check if it contains a solid block
//! 5. Stop when hitting a solid block or exceeding the Chebyshev distance limit
//!
//! # Distance Metric
//! Distance is measured in Chebyshev (max) distance: max(|dx|, |dy|, |dz|) blocks from origin.
//! This provides predictable raycast ranges and matches the voxel grid structure.
//!
//! # Coordinate Systems
//! - All coordinates are in world-space (IVec3)
//! - Chunk boundaries are at multiples of CHUNK_SIZE (64)
//! - Chunk-local coordinates are computed via rem_euclid for proper wrapping

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::map::{Block, CHUNK_SIZE, Chunk, ChunkData, ChunkId};

const DEFAULT_RAYCAST_DISTANCE_BLOCKS: u32 = 5;
const HALF_CHUNK_I32: i32 = CHUNK_SIZE as i32 / 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolidHitMode {
    StopOnSolid,
    ContinueThroughSolid,
}

#[derive(SystemParam)]
pub struct Raycast<'w, 's> {
    pub lookup: Res<'w, crate::map::ChunkLookup>,
    chunks: Query<'w, 's, &'static crate::map::Chunk>,
    data: Res<'w, Assets<ChunkData>>,
    pub world_offset: Res<'w, ChunkId>,
}

impl<'w, 's> Raycast<'w, 's> {
    /// gets the position for the first solid block hit by a ray (world-space coordinates)
    pub fn get_hit(&self, origin: &Transform) -> Option<IVec3> {
        self.get_hit_with_distance(origin, DEFAULT_RAYCAST_DISTANCE_BLOCKS)
    }

    /// gets the position for the first solid block hit by a ray with specified distance
    pub fn get_hit_with_distance(&self, origin: &Transform, max_distance: u32) -> Option<IVec3> {
        let mut state = RayState::new(origin, max_distance);
        self.traverse_ray(&mut state)
    }

    /// gets the path of the ray until it hits a solid block (world-space coordinates)
    pub fn cast_ray(&self, origin: &Transform) -> Vec<IVec3> {
        self.cast_ray_with_distance(origin, DEFAULT_RAYCAST_DISTANCE_BLOCKS)
    }

    /// gets the path of the ray until it hits a solid block with specified distance
    pub fn cast_ray_with_distance(&self, origin: &Transform, max_distance: u32) -> Vec<IVec3> {
        self.cast_ray_with_options(origin, max_distance, SolidHitMode::StopOnSolid)
    }

    /// gets the path of the ray with specified distance and solid-hit behavior
    pub fn cast_ray_with_options(
        &self,
        origin: &Transform,
        max_distance: u32,
        solid_hit_mode: SolidHitMode,
    ) -> Vec<IVec3> {
        let mut state = RayState::new(origin, max_distance);
        let mut all_blocks = Vec::new();
        let stop_on_solid = solid_hit_mode == SolidHitMode::StopOnSolid;

        loop {
            let chunk_id = state.chunk_id;

            // Get the chunk entity and data
            let Some(chunk_entity) = self.lookup.get(&chunk_id) else {
                break; // Ray exited loaded chunks
            };
            let Ok(chunk) = self.chunks.get(chunk_entity) else {
                break;
            };
            let Some(chunk_data) = self.data.get(&chunk.data) else {
                break;
            };

            // Traverse this chunk
            match self.update_ray(&mut state, chunk_data, stop_on_solid) {
                RaycastResult::Hit(_) => {
                    all_blocks.extend(state.chunk_path.drain(..));
                    break;
                }
                RaycastResult::Next(next_chunk_id) => {
                    // Add blocks traversed in this chunk to path
                    all_blocks.extend(state.chunk_path.drain(..));
                    state.chunk_id = next_chunk_id;
                }
                RaycastResult::Done => {
                    // Add any remaining blocks
                    all_blocks.extend(state.chunk_path.drain(..));
                    break;
                }
            }
        }

        all_blocks
    }

    /// Traverse ray through a single chunk, collecting blocks until hitting a solid block or exiting chunk
    fn update_ray(
        &self,
        state: &mut RayState,
        chunk: &ChunkData,
        stop_on_solid: bool,
    ) -> RaycastResult {
        loop {
            // Get block at current position
            let (local_x, local_y, local_z) = state.get_local_coords();

            if let Some(block) = chunk.get_block_checked(local_x, local_y, local_z) {
                // Add to path
                state.chunk_path.push(state.current);

                // Check if solid
                if block.is_solid() && stop_on_solid {
                    return RaycastResult::Hit(state.current);
                }

                // Step to next voxel
                match state.step_ray() {
                    Some((new_chunk_id, exceeded)) => {
                        // Crossed a boundary
                        if exceeded {
                            // Distance limit exceeded
                            return RaycastResult::Done;
                        } else {
                            // Moved to new chunk
                            return RaycastResult::Next(new_chunk_id);
                        }
                    }
                    None => {
                        // Continue in same chunk
                    }
                }
            } else {
                // Invalid local coordinates (shouldn't happen, indicates chunk transition)
                match state.step_ray() {
                    Some((new_chunk_id, exceeded)) => {
                        if exceeded {
                            return RaycastResult::Done;
                        } else {
                            return RaycastResult::Next(new_chunk_id);
                        }
                    }
                    None => {
                        // Continue (shouldn't happen)
                    }
                }
            }
        }
    }

    /// Internal helper to get first hit
    fn traverse_ray(&self, state: &mut RayState) -> Option<IVec3> {
        loop {
            let chunk_id = state.chunk_id;

            let Some(chunk_entity) = self.lookup.get(&chunk_id) else {
                return None;
            };
            let Ok(chunk) = self.chunks.get(chunk_entity) else {
                return None;
            };
            let Some(chunk_data) = self.data.get(&chunk.data) else {
                return None;
            };

            match self.update_ray(state, chunk_data, true) {
                RaycastResult::Hit(block_pos) => return Some(block_pos),
                RaycastResult::Next(next_chunk_id) => {
                    state.chunk_id = next_chunk_id;
                    state.chunk_path.clear();
                }
                RaycastResult::Done => return None,
            }
        }
    }
}

#[derive(Debug)]
struct RayState {
    // Origin and direction
    origin: Vec3,
    direction: Vec3,

    // Current block position in world space
    current: IVec3,

    // Current chunk
    chunk_id: ChunkId,

    // DDA state - stepping direction for each axis
    step_x: i32,
    step_y: i32,
    step_z: i32,

    // DDA state - time deltas (distance to cross one voxel boundary on each axis)
    t_delta_x: f32,
    t_delta_y: f32,
    t_delta_z: f32,

    // DDA state - accumulated time to next grid crossing
    t_max_x: f32,
    t_max_y: f32,
    t_max_z: f32,

    // Distance limit in Chebyshev blocks
    max_chebyshev: u32,
    current_chebyshev: u32,

    // Blocks collected in current chunk
    chunk_path: Vec<IVec3>,
}

impl RayState {
    /// Create a new ray state from transform with Chebyshev distance limit in blocks
    pub fn new(origin: &Transform, max_chebyshev_distance: u32) -> Self {
        let origin_pos = origin.translation;
        let direction = origin.forward();

        // Start at floored origin block
        let current = origin_pos.floor().as_ivec3();
        let chunk_id = ChunkId::from_block_position(Self::to_chunk_space_block(current));

        // Initialize DDA stepping based on ray direction
        let (step_x, t_delta_x, t_max_x) =
            Self::compute_dda_axis(direction.x, origin_pos.x, current.x);
        let (step_y, t_delta_y, t_max_y) =
            Self::compute_dda_axis(direction.y, origin_pos.y, current.y);
        let (step_z, t_delta_z, t_max_z) =
            Self::compute_dda_axis(direction.z, origin_pos.z, current.z);

        RayState {
            origin: origin_pos,
            direction: direction.normalize(),
            current,
            chunk_id,
            step_x,
            step_y,
            step_z,
            t_delta_x,
            t_delta_y,
            t_delta_z,
            t_max_x,
            t_max_y,
            t_max_z,
            max_chebyshev: max_chebyshev_distance,
            current_chebyshev: 0,
            chunk_path: Vec::new(),
        }
    }

    /// Compute DDA parameters for one axis
    /// Returns (step, t_delta, t_max)
    fn compute_dda_axis(
        dir_component: f32,
        origin_component: f32,
        block_coord: i32,
    ) -> (i32, f32, f32) {
        if dir_component.abs() < 1e-6 {
            // Ray doesn't move on this axis
            (0, f32::INFINITY, f32::INFINITY)
        } else {
            let step = if dir_component > 0.0 { 1 } else { -1 };
            let t_delta = 1.0 / dir_component.abs();

            // Compute distance to next grid line
            let grid_boundary = if dir_component > 0.0 {
                (block_coord as f32 + 1.0) - origin_component
            } else {
                origin_component - block_coord as f32
            };

            let t_max = t_delta * grid_boundary;
            (step, t_delta, t_max)
        }
    }

    /// Get local coordinates (chunk-relative) from current world position
    fn get_local_coords(&self) -> (u8, u8, u8) {
        let local =
            Self::to_chunk_space_block(self.current).rem_euclid(IVec3::splat(CHUNK_SIZE as i32));
        (local.x as u8, local.y as u8, local.z as u8)
    }

    #[inline(always)]
    fn to_chunk_space_block(block: IVec3) -> IVec3 {
        block + IVec3::splat(HALF_CHUNK_I32)
    }

    /// Advance ray to next voxel using DDA
    /// Returns (new_chunk_id_if_boundary_crossed, exceeded_distance_limit)
    /// If boundary crossed, new_chunk_id is returned
    /// If no crossing but distance limit exceeded, returns (current_chunk, true)
    /// Otherwise returns (current_chunk, false) - continue in same chunk
    fn step_ray(&mut self) -> Option<(ChunkId, bool)> {
        // Find which axis to step (minimum t_max)
        let min_t = self.t_max_x.min(self.t_max_y).min(self.t_max_z);

        // Step on the axis with minimum t_max
        if (self.t_max_x - min_t).abs() < 1e-6 && self.step_x != 0 {
            self.current.x += self.step_x;
            self.t_max_x += self.t_delta_x;
        }
        if (self.t_max_y - min_t).abs() < 1e-6 && self.step_y != 0 {
            self.current.y += self.step_y;
            self.t_max_y += self.t_delta_y;
        }
        if (self.t_max_z - min_t).abs() < 1e-6 && self.step_z != 0 {
            self.current.z += self.step_z;
            self.t_max_z += self.t_delta_z;
        }

        // Check distance limit (Chebyshev)
        self.current_chebyshev = Self::chebyshev_dist(self.origin.as_ivec3(), self.current);

        if self.current_chebyshev > self.max_chebyshev {
            return Some((self.chunk_id, true)); // Distance exceeded
        }

        // Check if we crossed chunk boundary
        let new_chunk_id = ChunkId::from_block_position(Self::to_chunk_space_block(self.current));
        if new_chunk_id != self.chunk_id {
            Some((new_chunk_id, false)) // Chunk boundary crossed
        } else {
            None // Continue in same chunk
        }
    }

    /// Compute Chebyshev distance between two points
    fn chebyshev_dist(a: IVec3, b: IVec3) -> u32 {
        let diff = (a - b).abs();
        diff.max_element() as u32
    }
}

enum RaycastResult {
    Hit(IVec3),
    Next(ChunkId),
    Done,
}

pub fn get_block_type(ray: &Raycast, block: IVec3) -> Block {
    let chunk_space_block = block + IVec3::splat(HALF_CHUNK_I32);
    let chunk = ChunkId::from_block_position(chunk_space_block);
    let Some(chunk_entity) = ray.lookup.get(&chunk) else {
        return Block::Void; // Treat out-of-chunk as air
    };
    let Ok(chunk) = ray.chunks.get(chunk_entity) else {
        return Block::Void;
    };
    let Some(chunk_data) = ray.data.get(&chunk.data) else {
        return Block::Void;
    };
    chunk_data
        .get_block_checked(
            chunk_space_block.x.rem_euclid(CHUNK_SIZE as i32) as u8,
            chunk_space_block.y.rem_euclid(CHUNK_SIZE as i32) as u8,
            chunk_space_block.z.rem_euclid(CHUNK_SIZE as i32) as u8,
        )
        .unwrap_or(Block::Void)
}
