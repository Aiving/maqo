use super::TextureLoader;
use crate::util::string::StrExt;
use data::model::Model;
use std::{collections::HashMap, fs};

#[derive(Default)]
pub struct BlockLoader {
    blocks: HashMap<String, Model>,
}

impl BlockLoader {
    pub fn get(&self, name: &str) -> Option<&Model> {
        self.blocks.get(name)
    }

    pub fn available(&self) -> Vec<&String> {
        self.blocks.keys().collect()
    }

    pub fn load(&mut self, textures: &mut TextureLoader, name: &str) -> Option<&Model> {
        let name = name.strip_id();
        let path = crate::asset!("models/{name}.json");

        let mut block: Model = serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|_| panic!("there is no model called {path}")),
        )
        .expect("can't parse block model");

        let parent = if let Some(parent) = block.parent.as_ref() {
            if let Some(model) = self.blocks.get(parent) {
                Some(model)
            } else {
                self.load(textures, parent)
            }
        } else {
            None
        };

        if let Some(parent) = parent {
            block.textures.extend(parent.textures.clone());

            if block.elements.is_empty() {
                block.elements = parent.elements.clone();
            }
        }

        for id in block.textures.values() {
            if let Some(id) = id.strip_prefix('#') {
                if let Some(texture) = block.textures.get(id) {
                    if !textures.contains(texture) {
                        textures.load(texture);
                    }
                }
            } else if !textures.contains(id) {
                textures.load(id);
            }
        }

        self.blocks.insert(format!("minecraft:{name}"), block);

        self.blocks.get(&format!("minecraft:{name}"))
    }
}
