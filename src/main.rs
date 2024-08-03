pub mod block;
pub mod block_states;
pub mod chunk;
pub mod data;
pub mod physics;
pub mod util;

use block::{Face, Model, Opacity, PartialModel};
use block_states::BlockStates;
use chunk::{Chunk, ChunkManager};
use indexmap::IndexMap;
use macroquad::{models::Vertex, prelude::*};
use miniquad::gl;
use noise::{NoiseFn, SuperSimplex};
use std::{array, collections::HashMap, env::current_dir, fs};
use util::{string::StrExt, vectors::Vec4Ext};

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
    blocks: HashMap<String, data::model::Model>,
    block_states: HashMap<String, data::block_states::BlockStates>,
    textures: HashMap<String, Texture2D>,
    textures_alpha: HashMap<String, u8>,
    world: ChunkManager,
}

impl Minecraft {
    fn get_alpha(&mut self, texture: &str) -> u8 {
        let texture = texture.as_id();

        if let Some(&alpha) = self.textures_alpha.get(&texture) {
            alpha
        } else {
            let alpha = self
                .textures
                .get(&texture)
                .and_then(|texture| {
                    texture
                        .get_texture_data()
                        .get_image_data()
                        .iter()
                        .map(|pixel| pixel[3])
                        .min()
                })
                .unwrap_or(0);

            self.textures_alpha.insert(texture, alpha);

            alpha
        }
    }

    fn create_world(&mut self, blocks: &[SimpleBlock]) {
        for block in blocks {
            let model = self.load_model(&block.tints, &block.name);

            // self.world.insert(block.position.as_i64vec3(), model);
        }
    }

    fn get_world_mesh(
        &self,
        position: IVec3,
        chunks: [[[&Chunk; 3]; 3]; 3],
    ) -> Vec<(&I64Vec3, Mesh)> {
        // let mut meshes = vec![];

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
                }
            }
        }

        // for face in todo!() {
        //     if face
        //         .cull_face
        //         .as_ref()
        //         .and_then(|cull_face| self.world.get(&(*position + cull_face.as_vec3())))
        //         .is_some_and(|neighbour| neighbour.opacity.is_opaque())
        //     {
        //         continue;
        //     }

        /*                 let vertices = face
                           .vertices
                           .into_iter()
                           .map(|vertex| {
                               let (rgb, mut num_colors) =
                                   (vertex.color, if vertex.color == WHITE { 0.0 } else { 1.0 });
                               let (mut sum_light_level, mut num_light_level) = (0.0, 0.0);

                               let [dx, dy, dz] = vertex.position.to_array().map(|x| x.round() as i32);

                               for dx in [dx - 1, dx] {
                                   for dz in [dz - 1, dz] {
                                       for dy in [dy - 1, dy] {
                                           let (neighbor, light_level) = self.get_block([dx, dy, dz]);
                                           let light_level =
                                               max(light_level.block_light(), light_level.sky_light());
                                           let mut light_level = light_level as f32;

                                           let use_block = match face.ao_face {
                                               Some(ao_face) => {
                                                   let mut above = true;

                                                   for (i, a) in
                                                       ao_face.as_vec3().to_array().into_iter().enumerate()
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
                                                       && block_states.get_opacity(neighbor).is_solid()
                                                   {
                                                       light_level = 0.0;
                                                   }

                                                   above
                                               }
                                               None => !block_states.get_opacity(neighbor).is_opaque(),
                                           };

                                           if use_block {
                                               sum_light_level += light_level;
                                               num_light_level += 1.0;
                                           }
                                       }

                                       if block.tints == 0 {
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
                                       rgb = vec3_add(
                                           rgb,
                                           match tint_source {
                                               model::Tint::None | model::Tint::Redstone => continue,
                                               model::Tint::Grass => biome.grass_color,
                                               model::Tint::Foliage => biome.foliage_color,
                                           }
                                           .map(|x| x as f32 / 255.0),
                                       );
                                       num_colors += 1.0;
                                   }
                               }
                           })
                           .collect();
        */
        //     meshes.push((
        //         position,
        //         Mesh {
        //             vertices: face
        //                 .vertices
        //                 .into_iter()
        //                 .map(|mut vertex| {
        //                     vertex.position += position.as_vec3();

        //                     vertex
        //                 })
        //                 .collect(),
        //             indices: vec![0, 1, 2, 2, 3, 0],
        //             texture: self.textures.get(&(&face.texture).as_id()).cloned(),
        //         },
        //     ));
        // }

        todo!()
    }

    // fn get_block(&self, position: I64Vec3) -> Option<(&Block, &String)> {
    //     let model = self.world.get(&position);

    //     model.and_then(|model| {
    //         self.blocks
    //             .get(&model.name)
    //             .map(|block| (block, &model.name))
    //     })
    // }

    fn block_exists(&self, position: I64Vec3) -> bool {
        todo!()
    }

    // fn get_colliders(&self, position: &I64Vec3) -> Vec<Direction> {
    //     let position = *position;

    //     vec![
    //         self.get_block(position + i64vec3(0, 1, 0))
    //             .and_then(|(_, block)| {
    //                 option_bool(!block.contains("lever") && !block.contains("glass"))
    //             })
    //             .map(|_| Direction::Top),
    //         self.get_block(position - i64vec3(0, 1, 0))
    //             .and_then(|(_, block)| {
    //                 option_bool(!block.contains("lever") && !block.contains("glass"))
    //             })
    //             .map(|_| Direction::Bottom),
    //         self.get_block(position + i64vec3(0, 0, 1))
    //             .and_then(|(_, block)| {
    //                 option_bool(!block.contains("lever") && !block.contains("glass"))
    //             })
    //             .map(|_| Direction::Front),
    //         self.get_block(position - i64vec3(0, 0, 1))
    //             .and_then(|(_, block)| {
    //                 option_bool(!block.contains("lever") && !block.contains("glass"))
    //             })
    //             .map(|_| Direction::Back),
    //         self.get_block(position + i64vec3(1, 0, 0))
    //             .and_then(|(_, block)| {
    //                 option_bool(!block.contains("lever") && !block.contains("glass"))
    //             })
    //             .map(|_| Direction::Right),
    //         self.get_block(position - i64vec3(1, 0, 0))
    //             .and_then(|(_, block)| {
    //                 option_bool(!block.contains("lever") && !block.contains("glass"))
    //             })
    //             .map(|_| Direction::Left),
    //     ]
    //     .into_iter()
    //     .flatten()
    //     .collect()
    // }
}

struct SimpleBlock {
    position: Vec3,
    name: String,
    tints: Vec<Color>,
}

impl SimpleBlock {
    fn new(x: f32, y: f32, z: f32, name: &str) -> Self {
        Self {
            position: vec3(x, y, z),
            name: name.to_string(),
            tints: vec![],
        }
    }

    fn with_tints(x: f32, y: f32, z: f32, name: &str, tints: Vec<Color>) -> Self {
        Self {
            position: vec3(x, y, z),
            name: name.to_string(),
            tints,
        }
    }
}

struct Player {
    rotation: Vec3,
    is_on_ground: bool,
    is_sneaking: bool,
    is_sprinting: bool,
    is_flying: bool,
    position: Vec3,
    aabb: AABB,
    velocity: Vec3,
    acceleration: Vec3,
}

impl Player {
    pub fn get_colliding_block_coords(&self, game: &Minecraft) -> Option<Vec3> {
        let player_mins = &self.aabb.mins;
        let player_maxs = &self.aabb.maxs;

        let block_mins = ivec3(
            player_mins.x.floor() as i32,
            player_mins.y.floor() as i32,
            player_mins.z.floor() as i32,
        );
        let block_maxs = ivec3(
            player_maxs.x.floor() as i32,
            player_maxs.y.floor() as i32,
            player_maxs.z.floor() as i32,
        );

        // We query all the blocks around the player to check whether it's colliding with one of them
        let mut colliding_block = None;

        for y in block_mins.y..=block_maxs.y {
            for z in block_mins.z..=block_maxs.z {
                for x in block_mins.x..=block_maxs.x {
                    if game.block_exists(i64vec3(x as i64, y as i64, z as i64)) {
                        let block_aabb = get_block_aabb(&vec3(x as f32, y as f32, z as f32));

                        if self.aabb.intersects(&block_aabb) {
                            colliding_block = Some(vec3(x as f32, y as f32, z as f32));

                            break;
                        }
                    }
                }
            }
        }

        colliding_block
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

    let mut world = vec![];

    let noise_fn = SuperSimplex::new(0xFF0FE0);

    for x in -25..25 {
        for y in 0..9 {
            for z in -25..25 {
                let (x, y, z) = (x as f64, y as f64, z as f64);

                let noise = noise_fn.get([x / 30.0, y * 15.0, z / 30.0]) * 80.0 + 64.0 + y * 1.7;

                if noise < 100.0 {
                    if y == 8.0 {
                        world.push(SimpleBlock::with_tints(
                            x as f32,
                            y as f32,
                            z as f32,
                            "minecraft:block/grass_block",
                            vec![Color::from_hex(0x6D9930)],
                        ));
                    } else {
                        world.push(SimpleBlock::new(
                            x as f32,
                            y as f32,
                            z as f32,
                            "minecraft:block/dirt",
                        ));
                    }
                }
            }
        }
    }

    world.push(SimpleBlock::new(
        3.0,
        3.0,
        3.0,
        "minecraft:block/crafting_table",
    ));
    world.push(SimpleBlock::new(3.0, 3.0, -3.0, "minecraft:block/lever"));
    world.push(SimpleBlock::new(-1.0, 3.0, 2.0, "minecraft:block/lever"));
    world.push(SimpleBlock::new(
        0.0,
        3.0,
        -3.0,
        "minecraft:block/black_stained_glass",
    ));
    world.push(SimpleBlock::new(2.0, 3.0, -3.0, "minecraft:block/glass"));
    world.push(SimpleBlock::new(1.0, 3.0, -3.0, "minecraft:block/glass"));

    app.create_world(&world);

    let mut pending_chunks = vec![];

    app.world
        .each_chunk_and_neighbors(|position, chunks, biomes| {
            pending_chunks.push((position, chunks, biomes));
        });

    let mesh = pending_chunks
        .into_iter()
        .map(|(position, chunks, _)| app.get_world_mesh(position, chunks))
        .collect::<Vec<_>>();

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

        set_camera(&Camera3D {
            position,
            up,
            target: position + front,
            ..Default::default()
        });

        unsafe {
            gl::glEnable(gl::GL_CULL_FACE);
            gl::glCullFace(gl::GL_BACK);
            // gl::glFrontFace(gl::GL_CW);
        }

        for meshes in &mesh {
            for (block_pos, mesh) in meshes {
                let block_pos = block_pos.as_vec3();

                let view_pos = block_pos - position;

                let angle = view_pos.angle_between(front);

                let distance = position.distance(block_pos);

                if angle < 50.0f32.to_radians() && distance < 25.0 {
                    draw_mesh(mesh);
                }
            }
        }

        // for i in 0..6 {
        //     let a = &mesh.vertices[i * 4..(i * 4) + 4];
        //     let a = a.iter().map(|x| &x.position).collect::<Vec<_>>();

        //     for (i, p1) in a.iter().enumerate() {
        //         for p2 in &a {
        //             draw_line_3d(
        //                 (**p1) + i as f32,
        //                 (**p2) + i as f32,
        //                 match i {
        //                     0 => RED,
        //                     1 => GREEN,
        //                     2 => BLUE,
        //                     _ => VIOLET,
        //                 },
        //             );
        //         }
        //     }
        // }

        // draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), GREEN);
        // draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), BLUE);
        // draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), RED);

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
