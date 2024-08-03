use std::{borrow::Cow, env::current_dir, fs};

use macroquad::{
    math::{ivec3, IVec3},
    texture::Texture2D,
};

use crate::{block::Model, data, Minecraft};

pub struct BlockStates {
    pub models: Vec<ModelAndBehavior>,
    pub texture: Texture2D,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RandomOffset {
    None,
    XZ,
    XYZ,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Dir {
    Down,
    Up,
    North,
    South,
    West,
    East,

    // Some diagonal directions (used by redstone).
    UpNorth,
    UpSouth,
    UpWest,
    UpEast,
}

impl Dir {
    pub fn xyz(self) -> IVec3 {
        match self {
            Self::Down => ivec3(0, -1, 0),
            Self::Up => ivec3(0, 1, 0),
            Self::North => ivec3(0, 0, -1),
            Self::South => ivec3(0, 0, 1),
            Self::West => ivec3(-1, 0, 0),
            Self::East => ivec3(1, 0, 0),

            Self::UpNorth => ivec3(0, 1, -1),
            Self::UpSouth => ivec3(0, 1, 1),
            Self::UpWest => ivec3(-1, 1, 0),
            Self::UpEast => ivec3(1, 1, 0),
        }
    }
}

#[derive(Clone, Copy)]
pub enum PolymorphDecision {
    // Stop and use this block state ID for the model.
    PickBlockState(u16),

    // Each of these checks a condition and continues if true,
    // or jumps to the provided u8 'else' index otherwise.
    // Blocks are specified with a signed offset from the block itself.
    // The 'OrSolid' variants also check for any solid blocks.
    IfBlock(Dir, i8, u8),
    IfBlockOrSolid(Dir, i8, u8),
    //IfGroup(Dir, Group, u8),
    //IfGroupOrSolid(Dir, Group, u8)
}

struct Description {
    id: u16,
    name: &'static str,
    variant: Cow<'static, str>,
    random_offset: RandomOffset,
    polymorph_oracle: Vec<PolymorphDecision>,
}

#[derive(Clone)]
pub struct ModelAndBehavior {
    pub model: Model,
    pub random_offset: RandomOffset,
    pub polymorph_oracle: Vec<PolymorphDecision>,
}

impl ModelAndBehavior {
    pub fn empty() -> ModelAndBehavior {
        ModelAndBehavior {
            model: Model::empty(),
            random_offset: RandomOffset::None,
            polymorph_oracle: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.model.is_empty()
    }
}

impl Minecraft {
    pub fn get_block_with_state(&self, name: &str) -> Option<Model> {
        if let Some(states) = self.block_states.get(name) {
            match states {
                data::block_states::BlockStates::Variants(variants) => {

                },
                data::block_states::BlockStates::Multipart(_) => todo!(),
            }
        }

        None
    }

    pub fn load_block_states(&mut self, name: &str) -> Option<&BlockStates> {
        let path = crate::asset!("blockstates/{name}.json");

        let states: data::block_states::BlockStates = serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|_| panic!("there is no block states called {path}")),
        )
        .expect("can't parse block states");

        self.block_states.insert(name.to_string(), states);

        None
    }
}
