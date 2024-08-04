use super::{BlockLoader, Face, Opacity, PartialModelLoader, TextureLoader};
use data::model::Direction;
use macroquad::{color::Color, math::U16Vec2};

#[derive(Debug, Clone)]
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

    fn _rotate(&mut self, ix: usize, iy: usize, rot_mat: [i32; 4], uvlock: bool) {
        let (a, b, c, d) = (
            rot_mat[0] as f32,
            rot_mat[1] as f32,
            rot_mat[2] as f32,
            rot_mat[3] as f32,
        );

        for face in self.faces.iter_mut() {
            for vertex in face.vertices.iter_mut() {
                let xyz = &mut vertex.position;

                let (x, y) = (xyz[ix], xyz[iy]);

                xyz[ix] = a * x + b * y;
                xyz[iy] = c * x + d * y;
            }

            let fixup_cube_face = |f: Direction| {
                let (a, b, c, d) = (rot_mat[0], rot_mat[1], rot_mat[2], rot_mat[3]);
                let mut dir = f.as_vec3();
                let (x, y) = (dir[ix], dir[iy]);

                dir[ix] = a * x + b * y;
                dir[iy] = c * x + d * y;

                Direction::from_vec3(dir).unwrap()
            };

            face.cull_face = face.cull_face.map(fixup_cube_face);
            face.ao_face = face.ao_face.map(fixup_cube_face);

            if uvlock {
                // Skip over faces that are constant in the ix or iy axis.
                let xs = face.vertices.map(|v| v.position[ix]);

                if xs.map(|x| (x - xs[0]).abs() < f32::EPSILON) == [true, true, true, true] {
                    continue;
                }

                let ys = face.vertices.map(|v| v.position[iy]);

                if ys.map(|y| (y - ys[0]).abs() < f32::EPSILON) == [true, true, true, true] {
                    continue;
                }

                let uvs = face.vertices.map(|x| x.uv);
                let uv_min =
                    [0, 1].map(|i| (uvs[0][i]).min(uvs[1][i]).min(uvs[2][i]).min(uvs[3][i]));
                let temp = uv_min.map(|x| (x / 16.0).floor() * 16.0);
                let (u_base, v_base) = (temp[0], temp[1]);

                for vertex in face.vertices.iter_mut() {
                    let uv = &mut vertex.uv;
                    let (u, v) = (uv[0] - u_base - 8.0, uv[1] - v_base - 8.0);

                    uv[0] = a * u - b * v + 8.0 + u_base;
                    uv[1] = -c * u + d * v + 8.0 + v_base;
                }
            }
        }
    }

    pub fn rotate(&mut self, ix: usize, iy: usize, r: usize, uvlock: bool) {
        match r {
            0 => {}
            90 => self._rotate(ix, iy, [0, -1, 1, 0], uvlock),
            180 => self._rotate(ix, iy, [-1, 0, 0, -1], uvlock),
            270 => self._rotate(ix, iy, [0, 1, -1, 0], uvlock),
            _ => unreachable!(),
        }
    }
}

pub struct ModelLoader;

impl ModelLoader {
    pub fn load_rotated(
        blocks: &BlockLoader,
        textures: &TextureLoader,
        tints: &[Color],
        name: &str,
        rotation: U16Vec2,
        uvlock: bool,
    ) -> Model {
        let mut model = Self::load(blocks, textures, tints, name);

        model.rotate(2, 1, rotation.x.into(), uvlock);
        model.rotate(0, 2, rotation.y.into(), uvlock);

        model
    }

    pub fn load(
        blocks: &BlockLoader,
        textures: &TextureLoader,
        tints: &[Color],
        name: &str,
    ) -> Model {
        let partial = PartialModelLoader::load(blocks, tints, name);

        let mut faces = partial.faces;
        let mut full_faces = [Opacity::Transparent; 6];

        if partial.full_faces.len() >= 6 {
            for &i in partial.full_faces.iter() {
                let face = faces[i].cull_face.unwrap() as usize;

                if full_faces[face] == Opacity::Opaque {
                    continue;
                }

                let opacity = match textures.get_alpha(&faces[i].texture) {
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
