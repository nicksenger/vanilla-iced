use glam::Vec2;
use iced::widget::shader::wgpu;

use crate::yuv::Size;

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Instance {
    pub position: Vec2,
    pub image_dimensions: Vec2,
    pub scale: Vec2,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn frame(size: Size, image_dimensions: Size, target_size: Size) -> [Self; 1] {
        let scale = (
            image_dimensions.width / target_size.width,
            image_dimensions.height / target_size.height,
        )
            .into();
        let position = [
            -1.0,
            (target_size.height - size.height * 2.0) / target_size.height,
        ]
        .into();
        let image_dimensions = (image_dimensions.width, image_dimensions.height).into();

        [Self {
            position,
            image_dimensions,
            scale,
        }]
    }
}
