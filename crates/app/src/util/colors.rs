use std::{fs, io, path::Path};

use macroquad::{color::Color, texture::Texture2D};

pub struct ColorMap(Texture2D);

impl ColorMap {
    /// Creates a new `ColorMap` from path.
    pub fn from_path<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let img = Texture2D::from_file_with_format(&fs::read(path.as_ref())?, None);

        match img.size().to_array() {
            [256.0, 256.0] => Ok(ColorMap(img)),
            [w, h] => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "ColorMap expected 256x256, found {}x{} in '{}'",
                    w,
                    h,
                    path.as_ref().display()
                ),
            )),
        }
    }

    /// Gets RGB color from the color map.
    pub fn get(&self, x: f32, y: f32) -> Color {
        // Clamp to [0.0, 1.0].
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);

        // Scale y from [0.0, 1.0] to [0.0, x], forming a triangle.
        let y = x * y;

        // Origin is in the bottom-right corner.
        let x = ((1.0 - x) * 255.0) as u8;
        let y = ((1.0 - y) * 255.0) as u8;

        self.0.get_texture_data().get_pixel(x as u32, y as u32)
    }
}
