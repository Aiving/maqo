use std::array;

use macroquad::{color::{Color, WHITE}, models::Vertex};

use crate::{util::{string::StrExt, vectors::Vec4Ext}, Minecraft};

use super::Face;

#[derive(Clone)]
pub struct PartialModel {
    pub faces: Vec<Face>,
    pub full_faces: Vec<usize>,
    pub ambient_occlusion: bool,
}

impl Minecraft {
    pub fn load_partial_model(&self, tints: &[Color], name: &str) -> PartialModel {
        let name = name.as_id();

        let block = self.blocks.get(&name).unwrap_or_else(|| {
            panic!(
                "failed to get block, available blocks: {:#?}",
                self.blocks.keys().collect::<Vec<_>>()
            )
        });

        let mut model = PartialModel {
            faces: vec![],
            full_faces: vec![],
            ambient_occlusion: block.ambient_occlusion.unwrap_or(true),
        };

        for element in &block.elements {
            let is_full_cube = element.is_full_cube();
            let corner_vertices = element.corner_vertices();

            for (direction, face) in &element.faces {
                let index = direction.vertices();
                let texture = block.get_texture(&face.texture);
                let face_uvs = face.uv.unwrap_or_else(|| element.get_face_uvs(direction));

                let uvs = direction.uvs(face_uvs.normalized_uvs(16.0));

                let color = face
                    .tint_index
                    .and_then(|index| tints.get(index).copied())
                    .unwrap_or(WHITE);

                if face.cullface.as_ref() == Some(direction) && is_full_cube {
                    model.full_faces.push(model.faces.len());
                }

                model.faces.push(Face {
                    vertices: array::from_fn(|i| Vertex {
                        uv: uvs[i],
                        position: corner_vertices[index[i]] / 16.0,
                        color,
                    }),
                    cull_face: face.cullface,
                    ao_face: if element.rotation.is_none() {
                        Some(*direction)
                    } else {
                        None
                    },
                    texture,
                });
            }
        }

        model
    }
}