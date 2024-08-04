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
use indexmap::IndexMap;
use serde::Deserialize;

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
