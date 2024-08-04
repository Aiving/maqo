use crate::Minecraft;
use data::block_states::{Property, PropertyValue};
use macroquad::color::Color;
use maqo_macros::Block;
use std::collections::HashMap;

pub trait GetProperty<T> {
    fn get_property(&self, name: &str) -> Option<&T>;
}

#[derive(Debug)]
pub struct BaseBlock {
    pub properties: HashMap<String, Property>,
    pub is_full_block: bool,
    pub is_translucent: bool,
    pub is_full_cube: bool,
    pub is_opaque_cube: bool,
    pub tints: Vec<Color>,
}

impl GetProperty<bool> for BaseBlock {
    fn get_property(&self, name: &str) -> Option<&bool> {
        self.properties
            .get(name)
            .and_then(|value| match &value.value {
                PropertyValue::Boolean(value) => Some(value),
                _ => None,
            })
    }
}

impl GetProperty<i64> for BaseBlock {
    fn get_property(&self, name: &str) -> Option<&i64> {
        self.properties
            .get(name)
            .and_then(|value| match &value.value {
                PropertyValue::Number(value) => Some(value),
                _ => None,
            })
    }
}

impl GetProperty<String> for BaseBlock {
    fn get_property(&self, name: &str) -> Option<&String> {
        self.properties
            .get(name)
            .and_then(|value| match &value.value {
                PropertyValue::String(value) => Some(value),
                _ => None,
            })
    }
}

pub trait Block {
    fn properties(&self) -> HashMap<String, Property>;
    fn is_full_block(&self) -> bool;

    fn is_full_cube(&self) -> bool {
        true
    }

    fn is_opaque_cube(&self) -> bool {
        true
    }

    fn is_translucent(&self) -> bool;

    fn tints(&self) -> Vec<Color> {
        Vec::new()
    }
}

impl<T: Block> From<T> for BaseBlock {
    fn from(value: T) -> Self {
        Self {
            properties: value.properties(),
            is_full_block: value.is_full_block(),
            is_translucent: value.is_translucent(),
            is_full_cube: value.is_full_cube(),
            is_opaque_cube: value.is_opaque_cube(),
            tints: value.tints(),
        }
    }
}

#[derive(Default, Block)]
#[block(full_block = true)]
#[tint(0x6D9930)]
pub struct GrassBlock {
    pub snowy: bool,
}

#[derive(Default, Block)]
#[block(full_block = true)]
pub struct DirtBlock {
    pub snowy: bool,
}

#[derive(Block)]
#[block(full_cube = false, opaque_cube = false)]
struct AirBlock;

pub enum BlockRenderLayer {
    Solid,
    CutoutMipped,
    Translucent,
}

impl Minecraft {
    pub fn init(&mut self) {
        self.register_block(0, "air", AirBlock);
        self.register_block(1, "dirt", DirtBlock::default());
        self.register_block(2, "grass_block", GrassBlock::default());
    }
}
