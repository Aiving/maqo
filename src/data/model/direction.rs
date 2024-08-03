use super::{Axis, AxisDirection};
use macroquad::math::{i64vec3, vec2, I64Vec3, Vec2, Vec4};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Direction {
    #[serde(rename = "up")]
    Top,
    #[serde(rename = "down")]
    Bottom,
    #[serde(rename = "north")]
    Front,
    #[serde(rename = "south")]
    Back,
    #[serde(rename = "east")]
    Right,
    #[serde(rename = "west")]
    Left,
}

impl Direction {
    pub fn vertices(&self) -> [usize; 4] {
        use Direction::*;

        match self {
            Right => [7, 6, 5, 4],
            Top => [1, 2, 5, 6],
            Left => [3, 2, 1, 0],
            Front => [3, 4, 5, 2],
            Bottom => [7, 4, 3, 0],
            Back => [1, 6, 7, 0],
        }
    }

    pub fn uvs(
        &self,
        Vec4 {
            x: u1,
            y: v1,
            z: u2,
            w: v2,
        }: Vec4,
    ) -> [Vec2; 4] {
        use Direction::*;

        match self {
            Bottom => [vec2(u1, v2), vec2(u1, v1), vec2(u2, v1), vec2(u2, v2)],
            Top => [vec2(u1, v2), vec2(u1, v1), vec2(u2, v1), vec2(u2, v2)],
            Front => [vec2(u2, v2), vec2(u1, v2), vec2(u1, v1), vec2(u2, v1)],
            Back => [vec2(u1, v1), vec2(u2, v1), vec2(u2, v2), vec2(u1, v2)],
            Left => [vec2(u1, v2), vec2(u1, v1), vec2(u2, v1), vec2(u2, v2)],
            Right => [vec2(u1, v2), vec2(u1, v1), vec2(u2, v1), vec2(u2, v2)],
        }
    }

    pub fn get_axis(&self) -> Axis {
        use Direction::*;

        match self {
            Top | Bottom => Axis::Y,
            Front | Back => Axis::Z,
            Right | Left => Axis::X,
        }
    }

    pub fn get_axis_dir(&self) -> AxisDirection {
        use Direction::*;

        match self {
            Top | Front | Right => AxisDirection::Positive,
            Bottom | Back | Left => AxisDirection::Negative,
        }
    }

    pub fn as_vec3(&self) -> I64Vec3 {
        use Direction::*;

        match self {
            Top => i64vec3(0, 1, 0),
            Bottom => i64vec3(0, -1, 0),
            Front => i64vec3(0, 0, 1),
            Back => i64vec3(0, 0, -1),
            Right => i64vec3(1, 0, 0),
            Left => i64vec3(-1, 0, 0),
        }
    }

    pub fn rotate_around(self, axis: &Axis) -> Self {
        use Direction::*;

        match axis {
            Axis::X => match self {
                Right | Left => self,
                value => value.rotate_x(),
            },
            Axis::Y => match self {
                Top | Bottom => self,
                value => value.rotate_y(),
            },
            Axis::Z => match self {
                Front | Back => self,
                value => value.rotate_z(),
            },
        }
    }

    pub fn rotate_x(&self) -> Self {
        use Direction::*;

        match self {
            Top => Back,
            Bottom => Front,
            Front => Bottom,
            Back => Top,
            Right | Left => unreachable!(),
        }
    }

    pub fn rotate_y(&self) -> Self {
        use Direction::*;

        match self {
            Front => Right,
            Back => Left,
            Right => Back,
            Left => Front,
            Top | Bottom => unreachable!(),
        }
    }

    pub fn rotate_z(&self) -> Self {
        use Direction::*;

        match self {
            Top => Right,
            Bottom => Left,
            Right => Bottom,
            Left => Top,
            Front | Back => unreachable!(),
        }
    }
}
