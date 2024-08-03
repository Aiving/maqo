mod axis;
mod direction;
mod element;
mod rotation;

pub use self::{
    axis::{Axis, AxisDirection},
    direction::Direction,
    element::{Element, Face},
    rotation::Rotation,
};
use crate::{util::string::StrExt, Minecraft};
use indexmap::IndexMap;
use macroquad::texture::{FilterMode, Texture2D};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Model {
    pub parent: Option<String>,
    #[serde(rename = "ambientocclusion")]
    pub ambient_occlusion: Option<bool>,
    #[serde(default)]
    pub textures: IndexMap<String, String>,
    #[serde(default)]
    pub elements: Vec<Element>,
}

impl Model {
    pub fn get_texture<T: AsRef<str>>(&self, t: T) -> String {
        let t = t.as_ref();

        if let Some(id) = t.strip_prefix('#') {
            self.get_texture(id)
        } else if let Some(id) = self.textures.get(t) {
            self.get_texture(id)
        } else {
            t.to_string()
        }
    }
}

impl Minecraft {
    pub fn load_block_model(&mut self, name: &str) -> Option<&Model> {
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
                self.load_block_model(parent)
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
                    if !self.textures.contains_key(texture) {
                        let texture = texture.strip_id();

                        let path = crate::asset!("textures/{texture}.png");

                        let bytes = fs::read(path).expect("failed to read texture");
                        let data = Texture2D::from_file_with_format(&bytes[..], None);

                        data.set_filter(FilterMode::Nearest);

                        self.textures.insert(format!("minecraft:{texture}"), data);
                    }
                }
            } else if !self.textures.contains_key(id) {
                let texture = id.strip_id();

                let path = crate::asset!("textures/{texture}.png");

                let bytes = fs::read(path).expect("failed to read texture");
                let data = Texture2D::from_file_with_format(&bytes[..], None);

                data.set_filter(FilterMode::Nearest);

                self.textures.insert(format!("minecraft:{texture}"), data);
            }
        }

        self.blocks.insert(format!("minecraft:{name}"), block);

        self.blocks.get(&format!("minecraft:{name}"))
    }
}
