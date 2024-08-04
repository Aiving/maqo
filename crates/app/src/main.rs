pub mod block_states;
pub mod chunk;
pub mod loaders;
pub mod util;

use block_states::Block;
use chunk::{BiomeId, BlockState, Chunk, ChunkColumn, ChunkManager, LightLevel};
use data::model::Direction;
use loaders::{BiomeLoader, BlockLoader, BlockStatesLoader, TextureLoader};
use macroquad::{models, prelude::*};
use miniquad::gl;
use noise::{NoiseFn, SuperSimplex};
use std::{array, cell::RefCell, env::current_dir};
use util::string::StrExt;

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;

#[macro_export]
macro_rules! asset {
    ($($arg:tt)*) => {
        $crate::get_asset(format!($($arg)*))
    }
}

pub fn get_asset<T: std::fmt::Display>(path: T) -> String {
    format!("{}/assets/{path}", current_dir().unwrap().to_string_lossy())
}
fn array_16x16x16<T, const SIZE: usize, F>(mut f: F) -> [[[T; SIZE]; SIZE]; SIZE]
where
    F: FnMut(usize, usize, usize) -> T,
{
    array::from_fn(|y| -> [[T; SIZE]; SIZE] {
        array::from_fn(|z| -> [T; SIZE] { array::from_fn(|x| f(x, y, z)) })
    })
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("Macroquad"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[derive(Default)]
pub struct Minecraft {
    block_states: BlockStatesLoader,
    blocks: BlockLoader,
    textures: TextureLoader,
    world: ChunkManager,
}

impl Minecraft {
    pub fn register_block(&mut self, id: usize, name: &str, block: impl Block + 'static) {
        self.block_states
            .register_block(&self.blocks, &self.textures, id, name, block);
    }

    pub fn load_block_model(&mut self, name: &str) {
        self.blocks.load(&mut self.textures, name);
    }

    pub fn load_block_states(&mut self, name: &str) {
        self.block_states.load(name);
    }

    fn create_world(&mut self, seed: u32) {
        let noise_fn = SuperSimplex::new(seed);

        let mut chunks = vec![];

        chunks.push(Chunk {
            blocks: array_16x16x16::<BlockState, 16, _>(|x, y, z| {
                let (x, y, z) = (x as f64, y as f64, z as f64);
                let noise = noise_fn.get([x / 30.0, y * 15.0, z / 30.0]) * 80.0 + 64.0 + y * 1.7;

                if noise < 100.0 {
                    if y == 15.0 {
                        BlockState { value: 2 }
                    } else {
                        BlockState { value: 1 }
                    }
                } else {
                    BlockState { value: 0 }
                }
            }),
            light_levels: [[[LightLevel { value: 0 }; 16]; 16]; 16],
        });

        self.world.add_chunk_column(
            ivec2(0, 0),
            ChunkColumn {
                chunks,
                biomes: [[BiomeId { value: 1 }; 16]; 16],
                buffers: array::from_fn(|_| RefCell::new(Vec::new())),
            },
        );
    }

    fn get_world_mesh(
        &self,
        position: IVec3,
        buffer: &mut Vec<(Vec3, Mesh)>,
        biomes: &BiomeLoader,
        chunks: [[[&Chunk; 3]; 3]; 3],
        column_biomes: [[Option<&[[BiomeId; 16]; 16]>; 3]; 3],
    ) {
        let chunk_xyz = position.as_vec3() * 16.0;

        for y in 0..16usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    let at = |dir: IVec3| {
                        let (dx, dy, dz) = (dir[0] as usize, dir[1] as usize, dir[2] as usize);
                        let (x, y, z) = (
                            x.wrapping_add(dx).wrapping_add(16),
                            y.wrapping_add(dy).wrapping_add(16),
                            z.wrapping_add(dz).wrapping_add(16),
                        );

                        let chunk = chunks[x / 16][y / 16][z / 16];

                        let (x, y, z) = (x % 16, y % 16, z % 16);

                        (chunk.blocks[y][z][x], chunk.light_levels[y][z][x])
                    };

                    let this_block = at(ivec3(0, 0, 0)).0;

                    let (_, model) = match self.block_states.get_by_id(this_block.value.into()) {
                        Some((name, model)) => (name, model),
                        None => continue,
                    };

                    let block_xyz = Vec3::from_array([x, y, z].map(|x| x as f32)) + chunk_xyz;

                    for face in model.faces.iter() {
                        if let Some(cull_face) = face.cull_face {
                            let (neighbor, _) = at(cull_face.as_vec3());

                            if self
                                .block_states
                                .get_by_id(neighbor.value.into())
                                .is_some_and(|(_, model)| model.opacity.is_opaque())
                            {
                                continue;
                            }
                        }

                        let v = face.vertices.map(|vertex| {
                            // Average tint and light around the vertex.
                            let (mut rgb, mut num_colors) = (
                                vertex.color,
                                match vertex.color {
                                    WHITE => 0.0,
                                    _ => 1.0,
                                },
                            );
                            let (mut sum_light_level, mut num_light_level) = (0.0, 0.0);

                            let rounded_xyz = IVec3::from_array(
                                vertex.position.to_array().map(|x| x.round() as i32),
                            );

                            let (dx, dy, dz) = (rounded_xyz[0], rounded_xyz[1], rounded_xyz[2]);

                            for &dx in [dx - 1, dx].iter() {
                                for &dz in [dz - 1, dz].iter() {
                                    for &dy in [dy - 1, dy].iter() {
                                        let (neighbor, light_level) = at(ivec3(dx, dy, dz));
                                        let light_level =
                                            light_level.block_light().max(light_level.sky_light());
                                        let mut light_level = light_level as f32;

                                        let use_block = match face.ao_face {
                                            Some(ao_face) => {
                                                let mut above = true;
                                                for (i, &a) in
                                                    ao_face.as_vec3().to_array().iter().enumerate()
                                                {
                                                    let da = [dx, dy, dz][i];
                                                    let va = rounded_xyz[i];
                                                    let above_da = match a {
                                                        -1 => va - 1,
                                                        1 => va,
                                                        _ => da,
                                                    };
                                                    if da != above_da {
                                                        above = false;
                                                        break;
                                                    }
                                                }

                                                if above
                                                    && self
                                                        .block_states
                                                        .get_by_id(neighbor.value.into())
                                                        .is_some_and(|(_, model)| {
                                                            model.opacity.is_solid()
                                                        })
                                                {
                                                    light_level = 0.0;
                                                }

                                                above
                                            }
                                            None => !self
                                                .block_states
                                                .get_by_id(neighbor.value.into())
                                                .is_some_and(|(_, model)| {
                                                    model.opacity.is_opaque()
                                                }),
                                        };

                                        if use_block {
                                            sum_light_level += light_level;
                                            num_light_level += 1.0;
                                        }
                                    }

                                    if vertex.color == WHITE {
                                        continue;
                                    }

                                    let (x, z) = (
                                        x.wrapping_add(dx as usize).wrapping_add(16),
                                        z.wrapping_add(dz as usize).wrapping_add(16),
                                    );
                                    let biome = match column_biomes[z / 16][x / 16] {
                                        Some(biome) => biomes[biome[z % 16][x % 16]],
                                        None => continue,
                                    };

                                    rgb = Color::from_vec(match vertex.color {
                                        WHITE => continue,
                                        _ => WHITE.to_vec() + biome.grass_color.to_vec(),
                                        // model::Tint::Foliage => biome.foliage_color,
                                    });

                                    num_colors += 1.0;
                                }
                            }

                            let light_factor = 0.2
                                + if num_light_level != 0.0 {
                                    sum_light_level / num_light_level / 15.0 * 0.8
                                } else {
                                    0.0
                                };

                            // Up, North and South, East and West, Down have different lighting.
                            let light_factor = light_factor
                                * match face.ao_face {
                                    Some(ao_face) => match ao_face {
                                        Direction::Top => 1.0,
                                        Direction::Front | Direction::Back => 0.8,
                                        Direction::Right | Direction::Left => 0.6,
                                        Direction::Bottom => 0.5,
                                    },
                                    None => 1.0,
                                };

                            let [r, g, b] = [rgb.r, rgb.g, rgb.b]
                                .map(|x| x * light_factor / num_colors - 2.0 / 255.0);

                            models::Vertex {
                                position: block_xyz + vertex.position,
                                uv: vertex.uv,
                                // No clue why the difference of 2 exists.
                                color: Color { r, g, b, a: rgb.a },
                            }
                        });

                        buffer.push((
                            block_xyz,
                            Mesh {
                                vertices: v.into(),
                                indices: vec![0, 1, 2, 2, 3, 0],
                                texture: self.textures.get(&face.texture.as_id()).cloned(),
                            },
                        ))
                    }
                }
            }
        }
    }
}

/// Creates an AABB box at mins with a length of 1 in every dimension
pub fn get_block_aabb(mins: &Vec3) -> AABB {
    AABB::new(*mins, *mins + vec3(1.0, 1.0, 1.0))
}

#[derive(Debug, Copy, Clone)]
pub struct AABB {
    pub mins: Vec3,
    pub maxs: Vec3,
}

impl AABB {
    pub fn new(mins: Vec3, maxs: Vec3) -> Self {
        Self { mins, maxs }
    }

    pub fn ip_translate(&mut self, translation: &Vec3) {
        self.mins += *translation;
        self.maxs += *translation;
    }

    /// Checks whether this AABB is intersecting another one
    pub fn intersects(&self, other: &Self) -> bool {
        (self.mins.x < other.maxs.x && self.maxs.x > other.mins.x)
            && (self.mins.y < other.maxs.y && self.maxs.y > other.mins.y)
            && (self.mins.z < other.maxs.z && self.maxs.z > other.mins.z)
    }

    pub fn contains_point(&self, other: &Vec3) -> bool {
        (self.mins.x < other.x && self.maxs.x > other.x)
            && (self.mins.y < other.y && self.maxs.y > other.y)
            && (self.mins.z < other.z && self.maxs.z > other.z)
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut app = Minecraft::default();

    app.load_block_model("minecraft:block/dirt");
    app.load_block_model("minecraft:block/grass_block");
    app.load_block_model("minecraft:block/glass");
    app.load_block_model("minecraft:block/black_stained_glass");
    app.load_block_model("minecraft:block/crafting_table");
    app.load_block_model("minecraft:block/lever");

    app.load_block_states("dirt");
    app.load_block_states("grass_block");
    app.load_block_states("black_stained_glass");
    app.load_block_states("glass");

    app.init();

    app.create_world(0xFF0FE0);

    let mut pending_chunks = vec![];

    app.world
        .each_chunk_and_neighbors(|position, buffer, chunks, biomes| {
            pending_chunks.push((position, buffer, chunks, biomes));
        });

    let biomes = BiomeLoader::new();

    for (position, buffer, chunks, biome_columns) in pending_chunks {
        app.get_world_mesh(
            position,
            &mut buffer.borrow_mut(),
            &biomes,
            chunks,
            biome_columns,
        );
    }

    let mut x = 0.0;
    let mut switch = false;
    let bounds = 8.0;

    let world_up = vec3(0.0, 1.0, 0.0);
    let mut yaw: f32 = 1.18;
    let mut pitch: f32 = 0.0;

    let mut front = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let mut right = front.cross(world_up).normalize();
    let mut up = right.cross(front).normalize();

    let mut position = vec3(0.0, 1.0, 0.0);
    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = true;

    set_cursor_grab(grabbed);
    show_mouse(false);

    let mut camera = Camera3D {
        position,
        up,
        target: position + front,
        ..Default::default()
    };

    loop {
        let delta = get_frame_time();

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        if is_key_down(KeyCode::W) {
            position += front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::S) {
            position -= front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::A) {
            position -= right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::D) {
            position += right * MOVE_SPEED;
        }

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;

        last_mouse_position = mouse_position;

        if grabbed {
            yaw += mouse_delta.x * delta * LOOK_SPEED;
            pitch += mouse_delta.y * delta * -LOOK_SPEED;

            pitch = if pitch > 1.5 { 1.5 } else { pitch };
            pitch = if pitch < -1.5 { -1.5 } else { pitch };

            front = vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();

            right = front.cross(world_up).normalize();
            up = right.cross(front).normalize();

            x += if switch { 0.04 } else { -0.04 };
            if x >= bounds || x <= -bounds {
                switch = !switch;
            }
        }

        clear_background(LIGHTGRAY);

        // Going 3d!

        camera.position = position;
        camera.up = up;
        camera.target = position + front;

        set_camera(&camera);

        unsafe {
            gl::glEnable(gl::GL_CULL_FACE);
            gl::glCullFace(gl::GL_BACK);
        }

        let forward = (camera.position - camera.target).normalize();
        let right = camera.up.cross(forward);
        let projection_mat = camera.matrix();
        let view_mat = Mat4::from_cols(
            vec4(right.x, camera.up.x, forward.x, 0.0),
            vec4(right.y, camera.up.y, forward.y, 0.0),
            vec4(right.z, camera.up.z, forward.z, 0.0),
            vec4(
                -right.dot(camera.position),
                camera.up.dot(camera.position),
                forward.dot(camera.position),
                1.0,
            ),
        );

        let mut num_chunks: usize = 0;
        let mut num_sorted_chunks: usize = 0;
        let mut num_total_chunks: usize = 0;

        app.world.each_chunk(|position, _, buffer| {
            let buffer = buffer.borrow_mut();

            num_total_chunks += 1;

            let inf = f32::INFINITY;
            let mut bb_min = [inf, inf, inf];
            let mut bb_max = [-inf, -inf, -inf];
            let xyz = position.as_vec3() * 16.0;

            for &dx in [0.0, 16.0].iter() {
                for &dy in [0.0, 16.0].iter() {
                    for &dz in [0.0, 16.0].iter() {
                        let v = xyz + vec3(dx, dy, dz);
                        let xyzw = vec4(
                            view_mat.row(0).dot(vec4(v[0], v[1], v[2], 1.0)),
                            view_mat.row(1).dot(vec4(v[0], v[1], v[2], 1.0)),
                            view_mat.row(2).dot(vec4(v[0], v[1], v[2], 1.0)),
                            view_mat.row(3).dot(vec4(v[0], v[1], v[2], 1.0)),
                        );

                        let v = vec4(
                            projection_mat.row(0).dot(xyzw),
                            projection_mat.row(1).dot(xyzw),
                            projection_mat.row(2).dot(xyzw),
                            projection_mat.row(3).dot(xyzw),
                        );
                        let xyz = vec3(v[0], v[1], v[2]) * (1.0 / v[3]);

                        bb_min = array::from_fn(|i| bb_min[i].min(xyz[i]));
                        bb_max = array::from_fn(|i| bb_max[i].max(xyz[i]));
                    }
                }
            }

            let cull_bits: [bool; 3] = array::from_fn(|i| {
                let (min, max) = (bb_min[i], bb_max[i]);

                min.signum() == max.signum() && min.abs().min(max.abs()) >= 1.0
            });

            if !cull_bits.iter().any(|&cull| cull) {
                for (_, mesh) in buffer.iter() {
                    draw_mesh(mesh);
                }

                num_chunks += 1;

                if bb_min[0] < 0.0 && bb_max[0] > 0.0 || bb_min[1] < 0.0 && bb_max[1] > 0.0 {
                    num_sorted_chunks += 1;
                }
            }
        });

        // Back to screen space, render some text

        set_default_camera();

        unsafe {
            gl::glFrontFace(gl::GL_CCW);
            gl::glCullFace(gl::GL_FRONT);
        }

        draw_text(
            format!("First Person Camera | {} FPS", get_fps()).as_str(),
            10.0,
            20.0,
            30.0,
            BLACK,
        );

        draw_text(
            format!("X: {} Y: {} Z: {}", position.x, position.y, position.z).as_str(),
            10.0,
            48.0 + 18.0,
            30.0,
            BLACK,
        );
        draw_text(
            format!("Press <TAB> to toggle mouse grab: {}", grabbed).as_str(),
            10.0,
            48.0 + 42.0,
            30.0,
            BLACK,
        );

        next_frame().await
    }
}
