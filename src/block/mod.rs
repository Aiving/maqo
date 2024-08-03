mod partial;

pub use self::partial::PartialModel;
use crate::{data::model::Direction, Minecraft};
use macroquad::{color::Color, models::Vertex};

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

#[derive(Clone)]
pub struct Face {
    pub vertices: [Vertex; 4],
    pub cull_face: Option<Direction>,
    pub ao_face: Option<Direction>,
    pub texture: String,
}

#[derive(Clone)]
pub struct Model {
    pub faces: Vec<Face>,
    pub tints: usize,
    pub opacity: Opacity,
}

impl Model {
    pub fn empty() -> Model {
        Model {
            faces: Vec::new(),
            tints: 0,
            opacity: Opacity::Transparent,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}

impl Minecraft {
    pub fn load_model(&mut self, tints: &[Color], name: &str) -> Model {
        let partial = self.load_partial_model(tints, name);

        let mut faces = partial.faces;
        let mut full_faces = [Opacity::Transparent; 6];

        if partial.full_faces.len() >= 6 {
            for &i in partial.full_faces.iter() {
                let face = faces[i].cull_face.unwrap() as usize;

                if full_faces[face] == Opacity::Opaque {
                    continue;
                }

                let opacity = match self.get_alpha(&faces[i].texture) {
                    0 => Opacity::TransparentSolid,
                    255 => Opacity::Opaque,
                    _ => Opacity::TranslucentSolid,
                };

                if full_faces[face] < opacity {
                    full_faces[face] = opacity;
                }
            }
        }

        if !partial.ambient_occlusion {
            for face in faces.iter_mut() {
                face.ao_face = None;
            }
        } else if faces.iter().any(|f| f.ao_face.is_none()) {
            println!(
                "Warning: model {} uses AO but has faces which are unsuitable",
                name
            );
        }

        Model {
            faces,
            tints: tints.len(),
            opacity: *full_faces.iter().min().unwrap(),
        }
    }
}
