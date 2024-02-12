use glam::Vec2;

use crate::Size;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    sampling_factor: Vec2,
    size: Vec2,
    scale: Vec2,
}

impl Uniforms {
    pub fn new(
        size: Size,
        image_dimensions: Size,
        sampling_factor: f32,
        target_size: Size,
    ) -> Self {
        let image_dimensions: Vec2 = (image_dimensions.width, image_dimensions.height).into();
        let scale = (
            target_size.width / image_dimensions.x / 2.0 * image_dimensions.x,
            target_size.height / image_dimensions.y / 2.0 * image_dimensions.y,
        )
            .into();

        Self {
            sampling_factor: (sampling_factor, sampling_factor).into(),
            size: (size.width, size.height).into(),
            scale,
        }
    }
}
