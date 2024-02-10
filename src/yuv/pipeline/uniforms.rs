use glam::Vec2;

use crate::yuv::Size;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    size: Vec2,
    image_dimensions: Vec2,
    scale: Vec2,
}

impl Uniforms {
    pub fn new(size: Size, image_dimensions: Size, target_size: Size) -> Self {
        let image_dimensions: Vec2 = (image_dimensions.width, image_dimensions.height).into();

        let scale = (
            image_dimensions.x / target_size.width,
            image_dimensions.y / target_size.height,
        )
            .into();

        Self {
            size: (size.width, size.height).into(),
            image_dimensions,
            scale,
        }
    }
}