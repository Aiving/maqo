mod biome;
mod block;
mod block_states;
mod model;
mod partial_model;
mod texture;

pub use self::{
    biome::{Biome, BiomeLoader},
    block::BlockLoader,
    block_states::BlockStatesLoader,
    model::{Model, ModelLoader},
    partial_model::{PartialModel, PartialModelLoader},
    texture::TextureLoader,
};
use data::model::Direction;
use macroquad::models::Vertex;

#[derive(Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Opacity {
    Transparent,
    TranslucentSolid,
    TransparentSolid,
    Opaque,
}

impl Opacity {
    pub fn is_opaque(self) -> bool {
        self == Self::Opaque
    }

    pub fn is_solid(self) -> bool {
        self != Self::Transparent
    }
}

#[derive(Debug, Clone)]
pub struct Face {
    pub vertices: [Vertex; 4],
    pub cull_face: Option<Direction>,
    pub ao_face: Option<Direction>,
    pub texture: String,
}
