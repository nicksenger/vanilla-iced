mod pipeline;

use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Mutex;

use pipeline::Pipeline;

use iced::mouse;
use iced::widget::shader;
use iced::Rectangle;
use shader::wgpu;

use self::pipeline::Uniforms;
use crate::{Renderable, Size, Yuv};

pub struct Program {
    dimensions: Size,
    sampling_factor: f32,
    data: RefCell<Option<Renderable>>,
}

impl Program {
    pub fn new(yuv: Yuv) -> Self {
        let renderable = Renderable::from(yuv);

        Self {
            dimensions: renderable.dimensions(),
            sampling_factor: renderable.sampling_factor(),
            data: RefCell::new(Some(renderable)),
        }
    }

    pub fn update_frame(&mut self, yuv: Yuv) {
        *self.data.borrow_mut() = Some(yuv.into());
    }
}

impl<Message> shader::Program<Message> for Program {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive(Mutex::new(match self.data.borrow_mut().take() {
            Some(yuv) => State::Pending {
                yuv: yuv.into(),
                bounds,
            },

            _ => State::Prepared {
                bounds,
                image_dimensions: self.dimensions,
                sampling_factor: self.sampling_factor,
            },
        }))
    }
}

#[derive(Debug)]
pub struct Primitive(Mutex<State>);

#[derive(Debug)]
enum State {
    Pending {
        yuv: Renderable,
        bounds: Rectangle,
    },
    Prepared {
        image_dimensions: Size,
        sampling_factor: f32,
        bounds: Rectangle,
    },
}

impl State {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Pending { bounds, .. } | Self::Prepared { bounds, .. } => *bounds,
        }
    }

    fn image_dimensions(&self) -> Size {
        match self {
            Self::Prepared {
                image_dimensions, ..
            } => *image_dimensions,
            Self::Pending { yuv, .. } => yuv.dimensions(),
        }
    }

    fn sampling_factor(&self) -> f32 {
        match self {
            Self::Prepared {
                sampling_factor, ..
            } => *sampling_factor,
            Self::Pending { yuv, .. } => yuv.sampling_factor(),
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: Rectangle,
        target_size: iced::Size<u32>,
        scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        let Ok(mut state) = self.0.lock() else {
            return;
        };

        let size = Size::from(bounds.size()) * scale_factor;
        let target_size = Size {
            width: target_size.width as f32,
            height: target_size.height as f32,
        };

        match state.deref() {
            State::Pending { yuv, .. } => {
                if !storage.has::<Pipeline>() {
                    storage.store(Pipeline::new(
                        device,
                        format,
                        yuv.dimensions(),
                        target_size,
                        size,
                        scale_factor,
                    ));
                }

                let pipeline = storage.get_mut::<Pipeline>().expect("yuv pipeline");

                pipeline.update_uniforms(
                    queue,
                    &Uniforms::new(size, yuv.dimensions(), target_size, yuv.sampling_factor()),
                );
                pipeline.update_frame(queue, yuv);
                pipeline.update_vertices(queue, yuv.dimensions(), size, target_size, scale_factor)
            }

            State::Prepared {
                image_dimensions,
                sampling_factor,
                ..
            } => {
                let pipeline = storage.get_mut::<Pipeline>().expect("yuv pipeline");

                pipeline.update_uniforms(
                    queue,
                    &Uniforms::new(size, *image_dimensions, target_size, *sampling_factor),
                );
                pipeline.update_vertices(queue, *image_dimensions, size, target_size, scale_factor);
            }
        }

        *state = State::Prepared {
            bounds: state.bounds(),
            image_dimensions: state.image_dimensions(),
            sampling_factor: state.sampling_factor(),
        }
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _target_size: iced::Size<u32>,
        _viewport: Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

        if let Ok(state) = self.0.lock() {
            pipeline.render(target, encoder, state.bounds());
        }
    }
}
