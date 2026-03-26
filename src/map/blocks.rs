use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Block {
    Void = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    BedRock = 4,
    Snow = 5,
    Sand = 6,
    Water = 7,
    Other(u8),
}

impl Block {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Void | Block::Water)
    }
    pub fn id(&self) -> u32 {
        match self {
            Block::Void => 0,
            Block::Grass => 1,
            Block::Dirt => 2,
            Block::Stone => 3,
            Block::BedRock => 4,
            Block::Snow => 5,
            Block::Sand => 6,
            Block::Water => 7,
            Block::Other(val) => *val as u32,
        }
    }
    pub fn from_id(id: u32) -> Self {
        match id {
            0 => Block::Void,
            1 => Block::Grass,
            2 => Block::Dirt,
            3 => Block::Stone,
            4 => Block::BedRock,
            5 => Block::Snow,
            6 => Block::Sand,
            other => {
                if other & 0x80000000 != 0 {
                    Block::Other((other & 0x7FFFFFFF) as u8)
                } else {
                    error!("Invalid block id: {}", other);
                    Block::Void
                }
            }
        }
    }
    pub fn block_type(&self) -> u8 {
        match self {
            Block::Void => 0,
            Block::Grass => 1,
            Block::Dirt => 2,
            Block::Stone => 3,
            Block::BedRock => 4,
            Block::Snow => 5,
            Block::Sand => 6,
            Block::Water => 7,
            Block::Other(id) => *id,
        }
    }
}
