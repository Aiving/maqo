use crate::{chunk::BiomeId, util::colors::ColorMap};
use macroquad::color::Color;
use std::ops::Index;

#[derive(Copy, Clone)]
pub struct Biome {
    pub name: &'static str,
    pub temperature: f32,
    pub humidity: f32,
    pub grass_color: Color,
    pub foliage_color: Color,
}

pub struct BiomeLoader {
    biomes: Box<[Option<Biome>; 256]>,
}

impl Default for BiomeLoader {
    fn default() -> Self {
        Self {
            biomes: Box::new([None; 256]),
        }
    }
}

impl BiomeLoader {
    pub fn new() -> Self {
        let mut loader = Self::default();

        loader.init();

        loader
    }
    
    pub fn init(&mut self) {
        let grass_colors =
            ColorMap::from_path(crate::asset!("textures/colormap/grass.png")).unwrap();
        let foliage_colors =
            ColorMap::from_path(crate::asset!("textures/colormap/foliage.png")).unwrap();

        self.biomes[1] = Some(Biome {
            name: "plains",
            temperature: 0.8,
            humidity: 0.4,
            grass_color: grass_colors.get(0.8, 0.4),
            foliage_color: foliage_colors.get(0.8, 0.4),
        });
    }
}

impl Index<BiomeId> for BiomeLoader {
    type Output = Biome;

    fn index(&self, id: BiomeId) -> &Biome {
        self.biomes[id.value as usize].as_ref().unwrap()
    }
}
