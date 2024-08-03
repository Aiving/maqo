use super::{Direction, Rotation};
use indexmap::IndexMap;
use macroquad::math::{vec3, vec4, Vec3, Vec4};
use serde::Deserialize;

const ZERO: Vec3 = vec3(0.0, 0.0, 0.0);
const MAX: Vec3 = vec3(16.0, 16.0, 16.0);

#[derive(Debug, Deserialize, Clone)]
pub struct Face {
    pub uv: Option<Vec4>,
    pub texture: String,
    pub cullface: Option<Direction>,
    #[serde(default)]
    pub rotation: u16,
    #[serde(rename = "tintindex")]
    pub tint_index: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Element {
    pub from: Vec3,
    pub to: Vec3,
    pub rotation: Option<Rotation>,
    #[serde(default)]
    pub faces: IndexMap<Direction, Face>,
}

impl Element {
    pub fn is_full_cube(&self) -> bool {
        self.from == ZERO && self.to == MAX
    }

    pub fn corner_vertices(&self) -> [Vec3; 8] {
        let Vec3 {
            x: x1,
            y: y1,
            z: z1,
        } = self.from - 8.0;

        let Vec3 {
            x: x2,
            y: y2,
            z: z2,
        } = self.to - 8.0;

        let vertices = [
            vec3(x1, y1, z1), // 0: 0, 0, 0
            vec3(x1, y2, z1), // 1: 0, 1, 0
            vec3(x1, y2, z2), // 2: 0, 1, 1
            vec3(x1, y1, z2), // 3: 0, 0, 1
            vec3(x2, y1, z2), // 4: 1, 0, 1
            vec3(x2, y2, z2), // 5: 1, 1, 1
            vec3(x2, y2, z1), // 6: 1, 1, 0
            vec3(x2, y1, z1), // 7: 1, 0, 0
        ];

        self.rotation
            .as_ref()
            .map(|rotation| rotation.rotate_corners(vertices))
            .unwrap_or(vertices)
    }

    pub fn get_face_uvs(&self, direction: &Direction) -> Vec4 {
        use Direction::*;

        match direction {
            Bottom => vec4(self.from.x, 16.0 - self.to.z, self.to.x, 16.0 - self.from.z),
            Top => vec4(self.from.x, self.from.z, self.to.x, self.to.z),
            Front => vec4(
                16.0 - self.to.x,
                16.0 - self.to.y,
                16.0 - self.from.x,
                16.0 - self.from.y,
            ),
            Back => vec4(self.from.x, 16.0 - self.to.y, self.to.x, 16.0 - self.from.y),
            Left => vec4(self.from.z, 16.0 - self.to.y, self.to.z, 16.0 - self.from.y),
            Right => vec4(
                16.0 - self.to.z,
                16.0 - self.to.y,
                16.0 - self.from.z,
                16.0 - self.from.y,
            ),
        }
    }
}
