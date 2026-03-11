use bevy::{
    asset::RenderAssetUsages,
    image::{TextureFormatPixelInfo, Volume},
    mesh::PrimitiveTopology,
    render::render_resource::Extent3d,
};
use rand::{RngExt, SeedableRng};

pub const CHUNK_SIZE: usize = 32;

use super::*;

#[derive(Asset, TypePath)]
pub struct ChunkData {
    blocks: [u8; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

impl ChunkData {
    pub fn test() -> Self {
        let mut data = [0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let i = z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x;
                    data[i] = rand::random();
                }
            }
        }
        Self { blocks: data }
    }

    pub fn to_image(&self) -> Image {
        let mut image = Image::new_uninit(
            Extent3d {
                width: CHUNK_SIZE as u32,
                height: CHUNK_SIZE as u32,
                depth_or_array_layers: CHUNK_SIZE as u32,
            },
            bevy::render::render_resource::TextureDimension::D3,
            bevy::render::render_resource::TextureFormat::Rgba32Float,
            RenderAssetUsages::all(),
        );
        let data = vec![
            0;
            image.texture_descriptor.format.pixel_size().expect(
                "Failed to create Image: can't get pixel size for this TextureFormat"
            ) * image.texture_descriptor.size.volume()
        ];
        image.data = Some(data);

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let i = z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x;
                    let mut color = Color::hsl(self.blocks[i] as f32, 1., 0.5);
                    if self.blocks[i] == 0 {
                        color.set_alpha(0.);
                    }
                    image
                        .set_color_at_3d(x as u32, y as u32, z as u32, color)
                        .unwrap();
                }
            }
        }

        image
    }

    pub fn generate(pos: IVec3) -> (Self, Image) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(
            (pos.x as u64) << 40 | (pos.y as u64) << 20 | pos.z as u64,
        );
        let mut data = [0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let i = z * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + x;
                    data[i] = rng.random();
                }
            }
        }
        let res = Self { blocks: data };
        let image = res.to_image();
        (res, image)
    }
}

#[derive(Component)]
pub struct Chunk {
    pub data: Handle<ChunkData>,
    pub pos: IVec3,
}

pub fn make_baked_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut color = Vec::new();
    let mut indices = Vec::new();
    let mut uv = Vec::new();

    for z in 0..CHUNK_SIZE {
        vertices.extend([
            [0., 0., z as f32],
            [0., CHUNK_SIZE as f32, z as f32],
            [CHUNK_SIZE as f32, CHUNK_SIZE as f32, z as f32],
            [CHUNK_SIZE as f32, 0., z as f32],
        ]);
        vertices.extend([
            [0., 0., 1. + z as f32],
            [0., CHUNK_SIZE as f32, 1. + z as f32],
            [CHUNK_SIZE as f32, CHUNK_SIZE as f32, 1. + z as f32],
            [CHUNK_SIZE as f32, 0., 1. + z as f32],
        ]);
        normals.extend([[0., 0., 1.]; 4]);
        normals.extend([[0., 0., -1.]; 4]);
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
        let x = x as f32;
        vertices.extend([
            [x, 0., 0.],
            [x, 0., CHUNK_SIZE as f32],
            [x, CHUNK_SIZE as f32, CHUNK_SIZE as f32],
            [x, CHUNK_SIZE as f32, 0.],
        ]);
        vertices.extend([
            [1. + x, 0., 0.],
            [1. + x, 0., CHUNK_SIZE as f32],
            [1. + x, CHUNK_SIZE as f32, CHUNK_SIZE as f32],
            [1. + x, CHUNK_SIZE as f32, 0.],
        ]);
        normals.extend([[1., 0., 0.]; 4]);
        normals.extend([[-1., 0., 0.]; 4]);
        color.extend([[1., 0., 0., 1.]; 8]);
        uv.extend([[0., 0.]; 4]);
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
        let y = y as f32;
        vertices.extend([
            [0., y, 0.],
            [CHUNK_SIZE as f32, y, 0.],
            [CHUNK_SIZE as f32, y, CHUNK_SIZE as f32],
            [0., y, CHUNK_SIZE as f32],
        ]);
        vertices.extend([
            [0., 1. + y, 0.],
            [CHUNK_SIZE as f32, 1. + y, 0.],
            [CHUNK_SIZE as f32, 1. + y, CHUNK_SIZE as f32],
            [0., 1. + y, CHUNK_SIZE as f32],
        ]);
        normals.extend([[0., 1., 0.]; 4]);
        normals.extend([[0., -1., 0.]; 4]);
        color.extend([[0., 1., 0., 1.]; 8]);
        uv.extend([[1., 0.]; 4]);
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

pub fn make_baked_mesh_lod_2() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut color = Vec::new();
    let mut indices = Vec::new();
    let mut uv = Vec::new();

    for z in 0..CHUNK_SIZE / 2 {
        let z = z as f32 * 2.;
        vertices.extend([
            [0., 0., z],
            [0., CHUNK_SIZE as f32, z],
            [CHUNK_SIZE as f32, CHUNK_SIZE as f32, z],
            [CHUNK_SIZE as f32, 0., z],
        ]);
        let z = 2. + z;
        vertices.extend([
            [0., 0., z],
            [0., CHUNK_SIZE as f32, z],
            [CHUNK_SIZE as f32, CHUNK_SIZE as f32, z],
            [CHUNK_SIZE as f32, 0., z],
        ]);
        normals.extend([[0., 0., 1.]; 4]);
        normals.extend([[0., 0., -1.]; 4]);
    }
    for z in 0..CHUNK_SIZE / 2 {
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
    for x in 0..CHUNK_SIZE / 2 {
        let x = x as f32 * 2.;
        vertices.extend([
            [x, 0., 0.],
            [x, 0., CHUNK_SIZE as f32],
            [x, CHUNK_SIZE as f32, CHUNK_SIZE as f32],
            [x, CHUNK_SIZE as f32, 0.],
        ]);
        let x = 2. + x;
        vertices.extend([
            [x, 0., 0.],
            [x, 0., CHUNK_SIZE as f32],
            [x, CHUNK_SIZE as f32, CHUNK_SIZE as f32],
            [x, CHUNK_SIZE as f32, 0.],
        ]);
        normals.extend([[1., 0., 0.]; 4]);
        normals.extend([[-1., 0., 0.]; 4]);
        color.extend([[1., 0., 0., 1.]; 8]);
        uv.extend([[0., 0.]; 4]);
    }
    for x in 0..CHUNK_SIZE / 2 {
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
    for y in 0..CHUNK_SIZE / 2 {
        let y = y as f32 * 2.;
        vertices.extend([
            [0., y, 0.],
            [CHUNK_SIZE as f32, y, 0.],
            [CHUNK_SIZE as f32, y, CHUNK_SIZE as f32],
            [0., y, CHUNK_SIZE as f32],
        ]);
        let y = y + 2.;
        vertices.extend([
            [0., y, 0.],
            [CHUNK_SIZE as f32, y, 0.],
            [CHUNK_SIZE as f32, y, CHUNK_SIZE as f32],
            [0., y, CHUNK_SIZE as f32],
        ]);
        normals.extend([[0., 1., 0.]; 4]);
        normals.extend([[0., -1., 0.]; 4]);
        color.extend([[0., 1., 0., 1.]; 8]);
        uv.extend([[1., 0.]; 4]);
    }
    for y in 0..CHUNK_SIZE / 2 {
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
