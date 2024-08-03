use macroquad::math::Vec4;

pub trait Vec4Ext {
    fn normalized_uvs(self, factor: f32) -> Self;
}

impl Vec4Ext for Vec4 {
    fn normalized_uvs(self, factor: f32) -> Vec4 {
        self / factor
    }
}
