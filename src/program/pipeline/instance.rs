use glam::Vec2;
use iced::{widget::shader::wgpu, Rectangle};

use crate::Size;

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Instance {
    pub position: Vec2,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x2,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn frame(bounds: Rectangle, target_size: Size) -> [Self; 1] {
        let position = [
            -1.0 + (bounds.x / target_size.width) * 2.0,
            -1.0 + ((target_size.height - (bounds.height + bounds.y)) / target_size.height) * 2.0,
        ]
        .into();

        [Self { position }]
    }
}
