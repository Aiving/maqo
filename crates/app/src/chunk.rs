use macroquad::{
    math::{ivec2, ivec3, IVec2, IVec3, Vec3},
    models::Mesh,
};
use std::{cell::RefCell, collections::HashMap};
// use crate::Model;

#[derive(Copy, Clone)]
pub struct BlockState {
    pub value: u16,
}

pub const EMPTY_BLOCK: BlockState = BlockState { value: 0 };

#[derive(Copy, Clone)]
pub struct BiomeId {
    pub value: u8,
}

#[derive(Copy, Clone)]
pub struct LightLevel {
    pub value: u8,
}

impl LightLevel {
    pub fn block_light(self) -> u8 {
        self.value & 0xf
    }

    pub fn sky_light(self) -> u8 {
        self.value >> 4
    }
}

pub const SIZE: usize = 16;

#[derive(Copy, Clone)]
pub struct Chunk {
    pub blocks: [[[BlockState; SIZE]; SIZE]; SIZE],
    pub light_levels: [[[LightLevel; SIZE]; SIZE]; SIZE],
}

pub const EMPTY_CHUNK: &Chunk = &Chunk {
    blocks: [[[EMPTY_BLOCK; SIZE]; SIZE]; SIZE],
    light_levels: [[[LightLevel { value: 0xf0 }; SIZE]; SIZE]; SIZE],
};

pub struct ChunkColumn {
    pub chunks: Vec<Chunk>,
    pub biomes: [[BiomeId; SIZE]; SIZE],
    pub buffers: [RefCell<Vec<(Vec3, Mesh)>>; SIZE],
}

#[derive(Default)]
pub struct ChunkManager {
    chunk_columns: HashMap<IVec2, ChunkColumn>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn each_chunk_and_neighbors<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(
            /*coords:*/ IVec3,
            /*buffer:*/ &'a RefCell<Vec<(Vec3, Mesh)>>,
            /*chunks:*/ [[[&'a Chunk; 3]; 3]; 3],
            /*biomes:*/ [[Option<&'a [[BiomeId; SIZE]; SIZE]>; 3]; 3],
        ),
    {
        for &position in self.chunk_columns.keys() {
            let columns = [-1, 0, 1].map(|dz| {
                [-1, 0, 1].map(|dx| {
                    self.chunk_columns
                        .get(&ivec2(position.x + dx, position.y + dz))
                })
            });

            let central = columns[1][1].unwrap();

            for y in 0..central.chunks.len() {
                let chunks = [-1, 0, 1].map(|dy| {
                    let y = y as i32 + dy;
                    columns.map(|cz| {
                        cz.map(|cx| {
                            cx.and_then(|c| c.chunks[..].get(y as usize))
                                .unwrap_or(EMPTY_CHUNK)
                        })
                    })
                });

                f(
                    ivec3(position.x, y as i32, position.y),
                    &central.buffers[y],
                    chunks,
                    columns.map(|cz| cz.map(|cx| cx.map(|c| &c.biomes))),
                )
            }
        }
    }

    pub fn add_chunk_column(&mut self, position: IVec2, c: ChunkColumn) {
        self.chunk_columns.insert(position, c);
    }

    pub fn each_chunk<F>(&self, mut f: F)
    where
        F: FnMut(
            /* position: */ IVec3,
            /* chunk: */ &Chunk,
            /* buffer: */ &RefCell<Vec<(Vec3, Mesh)>>,
        ),
    {
        for (&position, c) in self.chunk_columns.iter() {
            for (y, (c, b)) in c.chunks.iter().zip(c.buffers.iter()).enumerate() {
                f(ivec3(position.x, y as i32, position.y), c, b)
            }
        }
    }
}
