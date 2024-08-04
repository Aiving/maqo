use super::{BlockLoader, Model, ModelLoader, TextureLoader};
use crate::{
    block_states::{BaseBlock, Block},
    util::string::StrExt,
};
use data::block_states::{BlockStates, Variant};
use macroquad::{math::u16vec2, rand};
use std::{collections::HashMap, fs};

#[derive(Default)]
pub struct BlockStatesLoader {
    block_states: HashMap<String, BlockStates>,
    alias: HashMap<usize, String>,
    registry: HashMap<String, (BaseBlock, Model)>,
}

impl BlockStatesLoader {
    pub fn get_name_by_id(&self, id: usize) -> Option<&str> {
        self.alias.get(&id).map(String::as_str)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&(BaseBlock, Model)> {
        self.registry.get(&name.as_id())
    }

    pub fn get_by_id(&self, id: usize) -> Option<&(BaseBlock, Model)> {
        self.alias
            .get(&id)
            .and_then(|alias| self.registry.get(alias))
    }

    pub fn register_block(
        &mut self,
        blocks: &BlockLoader,
        textures: &TextureLoader,
        id: usize,
        name: &str,
        block: impl Block + 'static,
    ) {
        let block = block.into();

        if let Some(model) = self.load_model_with_state(blocks, textures, name, &block) {
            self.alias.insert(id, name.to_string());
            self.registry.insert(name.into(), (block, model));
        }
    }

    pub fn load_model_with_state(
        &self,
        blocks: &BlockLoader,
        textures: &TextureLoader,
        name: &str,
        block: &BaseBlock,
    ) -> Option<Model> {
        let states = self.block_states.get(&name.as_id())?;

        let mut model = None;

        match states {
            data::block_states::BlockStates::Variants(variants) => {
                for (condition, variant) in variants {
                    if model.is_some() {
                        break;
                    }

                    if condition.properties.is_empty() {
                        model = Some(match variant {
                            Variant::One(value) => ModelLoader::load_rotated(
                                blocks,
                                textures,
                                &block.tints,
                                &value.model,
                                u16vec2(value.x, value.y),
                                value.uvlock,
                            ),
                            Variant::Many(values) => {
                                let index = rand::gen_range(0, values.len());

                                let value = &values[index];

                                ModelLoader::load_rotated(
                                    blocks,
                                    textures,
                                    &block.tints,
                                    &value.model,
                                    u16vec2(value.x, value.y),
                                    value.uvlock,
                                )
                            }
                        });
                    } else {
                        for (key, property) in &condition.properties {
                            let prop = block.properties.get(key)?;

                            if prop.value == property.value {
                                model = Some(match variant {
                                    Variant::One(value) => ModelLoader::load_rotated(
                                        blocks,
                                        textures,
                                        &block.tints,
                                        &value.model,
                                        u16vec2(value.x, value.y),
                                        value.uvlock,
                                    ),
                                    Variant::Many(values) => {
                                        let index = rand::gen_range(0, values.len());

                                        let value = &values[index];

                                        ModelLoader::load_rotated(
                                            blocks,
                                            textures,
                                            &block.tints,
                                            &value.model,
                                            u16vec2(value.x, value.y),
                                            value.uvlock,
                                        )
                                    }
                                });
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
            data::block_states::BlockStates::Multipart(_) => todo!(),
        }

        model
    }

    pub fn load(&mut self, name: &str) {
        let path = crate::asset!("blockstates/{name}.json");

        let states: BlockStates = serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|_| panic!("there is no block states called {path}")),
        )
        .expect("can't parse block states");

        self.block_states.insert(name.as_id(), states);
    }
}
