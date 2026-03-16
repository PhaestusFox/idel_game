use bevy::{
    asset::RenderAssetUsages,
    image::{TextureFormatPixelInfo, Volume},
    mesh::PrimitiveTopology,
    render::render_resource::Extent3d,
};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_OFFSET: Vec3 = Vec3::splat(CHUNK_SIZE as f32 * 0.5 - 1.0);
const STEP: f64 = 2. / CHUNK_SIZE as f64;
// pub const TLC: f32 = 31.9990024566650390625; // this is CHUNK_SIZE - 0.001 + 1 bit
pub const TRC: f32 = 1.;
pub const BLC: f32 = -1.;

use crate::map::map_gen::MapDescriptor;

use super::*;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct ChunkData {
    pub is_empty: bool,
    blocks: [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

impl ChunkData {
    pub fn test() -> Self {
        let mut data = Self::empty();

        for z in 0..CHUNK_SIZE as u8 {
            for y in 0..CHUNK_SIZE as u8 {
                for x in 0..CHUNK_SIZE as u8 {
                    if y < CHUNK_SIZE as u8 / 2 {
                        data.set_block(x, y, z, Block::Other(rand::random()));
                    } else {
                        data.set_block(x, y, z, Block::Void);
                    }
                }
            }
        }
        data
    }

    pub fn to_image(&self) -> Image {
        let mut image = Image::new_uninit(
            Extent3d {
                width: CHUNK_SIZE as u32,
                height: CHUNK_SIZE as u32,
                depth_or_array_layers: CHUNK_SIZE as u32,
            },
            bevy::render::render_resource::TextureDimension::D3,
            bevy::render::render_resource::TextureFormat::Rgba8Uint,
            RenderAssetUsages::all(),
        );
        let data = vec![
            0;
            image.texture_descriptor.format.pixel_size().expect(
                "Failed to create Image: can't get pixel size for this TextureFormat"
            ) * image.texture_descriptor.size.volume()
        ];
        image.data = Some(data);
        image.texture_descriptor.usage |=
            bevy::render::render_resource::TextureUsages::STORAGE_BINDING;

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = self.get_block(x as u8, y as u8, z as u8);
                    let Some(bytes) =
                        image.pixel_bytes_mut(UVec3::new(x as u32, y as u32, z as u32))
                    else {
                        error!("Failed to get pixel bytes for chunk image");
                        continue;
                    };
                    if bytes.len() == 4 {
                        bytes[0] = block.block_type();
                    } else {
                        panic!(
                            "Unexpected pixel byte length: expected 4, got {}",
                            bytes.len()
                        );
                    }
                }
            }
        }

        image
    }

    pub fn dummy_image() -> Image {
        let mut image = Image::new_uninit(
            Extent3d {
                width: CHUNK_SIZE as u32,
                height: CHUNK_SIZE as u32,
                depth_or_array_layers: CHUNK_SIZE as u32,
            },
            bevy::render::render_resource::TextureDimension::D3,
            bevy::render::render_resource::TextureFormat::Rgba8Uint,
            RenderAssetUsages::all(),
        );
        let data = vec![
            0;
            image.texture_descriptor.format.pixel_size().expect(
                "Failed to create Image: can't get pixel size for this TextureFormat"
            ) * image.texture_descriptor.size.volume()
        ];
        image.data = Some(data);
        image.texture_descriptor.usage |=
            bevy::render::render_resource::TextureUsages::STORAGE_BINDING;
        image
    }

    pub fn generate(pos: IVec3, map_descriptor: &MapDescriptor) -> (Self, Image) {
        let mut data = Self::empty();
        for z in 0..CHUNK_SIZE as u8 {
            for y in 0..CHUNK_SIZE as u8 {
                for x in 0..CHUNK_SIZE as u8 {
                    let block_pos =
                        pos * CHUNK_SIZE as i32 + IVec3::new(x as i32, y as i32, z as i32);
                    let block = map_descriptor.get_block(block_pos);
                    data.set_block(x, y, z, block);
                }
            }
        }
        let image = data.to_image();
        (data, image)
    }

    #[inline(always)]
    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: Block) {
        if block == Block::Void {
            return;
        }
        let i = Self::get_index(x, y, z);
        self.is_empty = false;
        self.blocks[i] = block;
    }

    pub fn empty() -> Self {
        Self {
            is_empty: true,
            blocks: [Block::Void; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    #[inline(always)]
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> Block {
        let i = Self::get_index(x, y, z);
        self.blocks[i]
    }

    #[inline(always)]
    pub fn get_index(x: u8, y: u8, z: u8) -> usize {
        z as usize * CHUNK_SIZE * CHUNK_SIZE + y as usize * CHUNK_SIZE + x as usize
    }
}

#[derive(Component)]
pub struct Chunk {
    pub is_empty: bool,
    pub data: Handle<ChunkData>,
}

#[derive(Component, Deref)]
pub struct ChunkId(IVec3);

impl ChunkId {
    pub fn new(pos: IVec3) -> Self {
        Self(pos)
    }

    pub fn offset(&self) -> Vec3 {
        (self.0 * CHUNK_SIZE as i32).as_vec3()
    }
}

pub fn make_baked_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    const CORRECTION_SCALE: f32 = CHUNK_SIZE as f32;
    const OFFSET: f32 = 0.01;

    for z in 0..CHUNK_SIZE {
        let o = (-1f64 + (STEP * z as f64)) as f32;
        let b = (-1f64 + (STEP * (z + 1) as f64)) as f32;
        vertices.extend([[BLC, BLC, o], [BLC, TRC, o], [TRC, TRC, o], [TRC, BLC, o]]);
        vertices.extend([[BLC, BLC, b], [BLC, TRC, b], [TRC, TRC, b], [TRC, BLC, b]]);
        let n = [
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., 1.],
            [0., 0., 1.],
            [0., 0., 1.],
            [0., 0., 1.],
        ]
        .map(|v| Vec3::from(v).normalize_or_zero().to_array());
        normals.extend(n);
    }
    for z in 0..CHUNK_SIZE {
        let offset = z as u16 * 8;
        indices.extend([
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
            offset + 4, // 0
            offset + 6, // 2
            offset + 5, // 1
            offset + 4, // 0
            offset + 7, // 3
            offset + 6, // 2
        ]);
    }
    let x_off = vertices.len() as u16;
    for x in 0..CHUNK_SIZE {
        let o = (-1f64 + (STEP * x as f64)) as f32;
        let b = (-1f64 + (STEP * (x + 1) as f64)) as f32;
        vertices.extend([[o, BLC, BLC], [o, BLC, TRC], [o, TRC, TRC], [o, TRC, BLC]]);
        vertices.extend([[b, BLC, BLC], [b, BLC, TRC], [b, TRC, TRC], [b, TRC, BLC]]);
        let n = [
            [-1., 0.0, 0.0],
            [-1., 0.0, 0.0],
            [-1., 0.0, 0.0],
            [-1., 0.0, 0.0],
            [1., 0.0, 0.0],
            [1., 0.0, 0.0],
            [1., 0.0, 0.0],
            [1., 0.0, 0.0],
        ]
        .map(|v| Vec3::from(v).normalize_or_zero().to_array());

        normals.extend(n);
    }
    for x in 0..CHUNK_SIZE {
        let offset = x_off + x as u16 * 8;
        indices.extend([
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
            offset + 4, // 0
            offset + 6, // 2
            offset + 5, // 1
            offset + 4, // 0
            offset + 7, // 3
            offset + 6, // 2
        ]);
    }
    let y_off = vertices.len() as u16;
    for y in 0..CHUNK_SIZE {
        let o = (-1f64 + (STEP * y as f64)) as f32;
        let b = (-1f64 + (STEP * (y + 1) as f64)) as f32;
        vertices.extend([[BLC, o, BLC], [TRC, o, BLC], [TRC, o, TRC], [BLC, o, TRC]]);
        vertices.extend([[BLC, b, BLC], [TRC, b, BLC], [TRC, b, TRC], [BLC, b, TRC]]);
        normals.extend(
            [
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]
            .map(|v| Vec3::from(v).normalize_or_zero().to_array()),
        );
    }
    for y in 0..CHUNK_SIZE {
        let offset = y_off + y as u16 * 8;
        indices.extend([
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
            offset + 4, // 0
            offset + 6, // 2
            offset + 5, // 1
            offset + 4, // 0
            offset + 7, // 3
            offset + 6, // 2
        ]);
    }

    mesh.insert_indices(bevy::mesh::Indices::U16(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}

pub fn make_baked_mesh_lod(lod: LoD) -> Mesh {
    if lod == LoD::LOD1 {
        return make_baked_mesh();
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let stride = CHUNK_SIZE / lod.step() as usize;

    for z in 0..stride {
        let o = (-1f64 + (STEP * z as f64 * lod.step() as f64)) as f32;
        let b = (-1f64 + (STEP * (z + 1) as f64 * lod.step() as f64)) as f32;
        vertices.extend([[BLC, BLC, o], [BLC, TRC, o], [TRC, TRC, o], [TRC, BLC, o]]);
        vertices.extend([[BLC, BLC, b], [BLC, TRC, b], [TRC, TRC, b], [TRC, BLC, b]]);
        let n = [
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., -1.],
            [0., 0., 1.],
            [0., 0., 1.],
            [0., 0., 1.],
            [0., 0., 1.],
        ]
        .map(|v| Vec3::from(v).normalize_or_zero().to_array());
        normals.extend(n);
    }
    for z in 0..stride {
        let offset = z as u16 * 8;
        indices.extend([
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
            offset + 4, // 0
            offset + 6, // 2
            offset + 5, // 1
            offset + 4, // 0
            offset + 7, // 3
            offset + 6, // 2
        ]);
    }
    let x_off = vertices.len() as u16;
    for x in 0..stride {
        let o = (-1f64 + (STEP * x as f64 * lod.step() as f64)) as f32;
        let b = (-1f64 + (STEP * (x + 1) as f64 * lod.step() as f64)) as f32;
        vertices.extend([[o, BLC, BLC], [o, BLC, TRC], [o, TRC, TRC], [o, TRC, BLC]]);
        vertices.extend([[b, BLC, BLC], [b, BLC, TRC], [b, TRC, TRC], [b, TRC, BLC]]);
        let n = [
            [-1., 0.0, 0.0],
            [-1., 0.0, 0.0],
            [-1., 0.0, 0.0],
            [-1., 0.0, 0.0],
            [1., 0.0, 0.0],
            [1., 0.0, 0.0],
            [1., 0.0, 0.0],
            [1., 0.0, 0.0],
        ]
        .map(|v| Vec3::from(v).normalize_or_zero().to_array());

        normals.extend(n);
    }
    for x in 0..stride {
        let offset = x_off + x as u16 * 8;
        indices.extend([
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
            offset + 4, // 0
            offset + 6, // 2
            offset + 5, // 1
            offset + 4, // 0
            offset + 7, // 3
            offset + 6, // 2
        ]);
    }
    let y_off = vertices.len() as u16;
    for y in 0..stride {
        let o = (-1f64 + (STEP * y as f64 * lod.step() as f64)) as f32;
        let b = (-1f64 + (STEP * (y + 1) as f64 * lod.step() as f64)) as f32;
        vertices.extend([[BLC, o, BLC], [TRC, o, BLC], [TRC, o, TRC], [BLC, o, TRC]]);
        vertices.extend([[BLC, b, BLC], [TRC, b, BLC], [TRC, b, TRC], [BLC, b, TRC]]);
        normals.extend(
            [
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]
            .map(|v| Vec3::from(v).normalize_or_zero().to_array()),
        );
    }
    for y in 0..stride {
        let offset = y_off + y as u16 * 8;
        indices.extend([
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 2,
            offset + 3,
            offset + 4, // 0
            offset + 6, // 2
            offset + 5, // 1
            offset + 4, // 0
            offset + 7, // 3
            offset + 6, // 2
        ]);
    }

    mesh.insert_indices(bevy::mesh::Indices::U16(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}
