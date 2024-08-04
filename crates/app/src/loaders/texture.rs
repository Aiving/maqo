use crate::util::string::StrExt;
use macroquad::texture::{FilterMode, Texture2D};
use std::{collections::HashMap, fs};

#[derive(Default)]
pub struct TextureLoader {
    textures: HashMap<String, (Texture2D, u8)>,
}

impl TextureLoader {
    pub fn contains(&self, texture: &str) -> bool {
        self.textures.contains_key(&texture.as_id())
    }

    pub fn load(&mut self, texture: &str) {
        let texture = texture.strip_id();

        let path = crate::asset!("textures/{texture}.png");

        let bytes = fs::read(path).expect("failed to read texture");
        let data = Texture2D::from_file_with_format(&bytes[..], None);

        data.set_filter(FilterMode::Nearest);

        let alpha = data
            .get_texture_data()
            .get_image_data()
            .iter()
            .map(|pixel| pixel[3])
            .min()
            .unwrap_or(0);

        self.textures.insert(texture.as_id(), (data, alpha));
    }

    pub fn get(&self, texture: &str) -> Option<&Texture2D> {
        self.textures
            .get(&texture.as_id())
            .map(|(texture, _)| texture)
    }

    pub fn get_alpha(&self, texture: &str) -> u8 {
        let texture = texture.as_id();

        if let Some((_, alpha)) = self.textures.get(&texture) {
            *alpha
        } else {
            0
        }
    }
}
