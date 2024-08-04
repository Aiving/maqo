use super::Axis;
use macroquad::math::{Mat3, Vec3};
use serde::Deserialize;
use std::f32::consts::{PI, SQRT_2};

#[derive(Debug, Deserialize, Clone)]
pub struct Rotation {
    pub origin: Vec3,
    pub angle: f32,
    pub axis: Axis,
    #[serde(default)]
    pub rescale: bool,
}

impl Rotation {
    pub fn as_mat3(&self) -> Mat3 {
        let angle = self.angle / 180.0 * PI;
        let scale = if self.rescale {
            SQRT_2 / (angle.cos().powi(2) * 2.0).sqrt()
        } else {
            1.0
        };

        let a = angle.cos() * scale;
        let b = angle.sin() * scale;

        match &self.axis {
            Axis::X => Mat3::from_cols_array(&[1.0, 0.0, 0.0, 0.0, a, -b, 0.0, b, a]),
            Axis::Y => Mat3::from_cols_array(&[a, 0.0, b, 0.0, 1.0, 0.0, -b, 0.0, a]),
            Axis::Z => Mat3::from_cols_array(&[a, -b, 0.0, b, a, 0.0, 0.0, 0.0, 1.0]),
        }
    }

    pub fn rotate_corners(&self, corners: [Vec3; 8]) -> [Vec3; 8] {
        let origin = self.origin - 8.0;
        let matrix = self.as_mat3();

        corners.map(|corner| matrix * (corner - origin) + origin)
    }
}
