use iced::widget::shader::wgpu;
use iced::{
    widget::shader::wgpu::util::{BufferInitDescriptor, DeviceExt},
    Rectangle,
};

mod instance;
mod uniforms;

use super::Frame;
use crate::yuv::Size;
use instance::Instance;
pub use uniforms::Uniforms;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniforms_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    texture_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    scale_factor: f32,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        image_dimensions: Size,
        target_size: Size,
        size: Size,
        scale_factor: f32,
    ) -> Self {
        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yuv uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("yuv uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Uniforms>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yuv uniform bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniforms_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("yuv texture"),
            size: wgpu::Extent3d {
                width: image_dimensions.width as u32,
                height: image_dimensions.height as u32,
                depth_or_array_layers: 3,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("yuv sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("yuv texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("yuv pipeline layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("yuv shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(concat!(include_str!(
                "shader.wgsl"
            ),))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu::image pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("yuv vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&Instance::frame(size, image_dimensions, target_size)),
        });

        Self {
            pipeline,
            uniforms_buffer,
            uniform_bind_group,
            texture,
            texture_bind_group,
            vertex_buffer,
            scale_factor,
        }
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, uniforms: &Uniforms) {
        queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    pub fn update_frame<T: AsRef<[u8]>>(&mut self, queue: &wgpu::Queue, frame: &Frame<T>) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::default(),
            },
            frame.y.as_ref(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(frame.strides.y as u32),
                rows_per_image: Some(frame.dimensions.y.height as u32),
            },
            wgpu::Extent3d {
                width: frame.dimensions.y.width as u32,
                height: frame.dimensions.y.height as u32,
                depth_or_array_layers: 1,
            },
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 1 },
                aspect: wgpu::TextureAspect::default(),
            },
            frame.u.as_ref(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(frame.strides.u as u32),
                rows_per_image: Some(frame.dimensions.u.height as u32),
            },
            wgpu::Extent3d {
                width: frame.dimensions.u.width as u32,
                height: frame.dimensions.u.height as u32,
                depth_or_array_layers: 1,
            },
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 2 },
                aspect: wgpu::TextureAspect::default(),
            },
            frame.v.as_ref(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(frame.strides.v as u32),
                rows_per_image: Some(frame.dimensions.v.height as u32),
            },
            wgpu::Extent3d {
                width: frame.dimensions.v.width as u32,
                height: frame.dimensions.v.height as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn update_vertices(
        &mut self,
        queue: &wgpu::Queue,
        image_dimensions: Size,
        size: Size,
        target_size: Size,
        scale_factor: f32,
    ) {
        self.scale_factor = scale_factor;
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::bytes_of(&Instance::frame(size, image_dimensions, target_size)),
        );
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        bounds: Rectangle,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("yuv.pipeline.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_scissor_rect(
            bounds.x as u32,
            bounds.y as u32,
            (bounds.width * self.scale_factor) as u32,
            (bounds.height * self.scale_factor) as u32,
        );

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, &self.texture_bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}
