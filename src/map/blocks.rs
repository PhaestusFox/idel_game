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
    IronOx = 8,
    Other(u8),
}

impl Block {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Void | Block::Water)
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
            Block::IronOx => 8,
            Block::Other(id) => *id,
        }
    }
}
