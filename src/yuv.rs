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

#[derive(Debug, Clone)]
pub struct Frame<T> {
    pub strides: Strides,
    pub dimensions: Dimensions,
    pub y: T,
    pub u: T,
    pub v: T,
}

#[derive(Debug, Clone)]
pub struct Strides {
    pub y: usize,
    pub u: usize,
    pub v: usize,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub y: Size,
    pub u: Size,
    pub v: Size,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl From<(f32, f32)> for Size {
    fn from(value: (f32, f32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<iced::Size<f32>> for Size {
    fn from(size: iced::Size<f32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

pub struct Program<T> {
    dimensions: Size,
    data: RefCell<Option<Frame<T>>>,
}

impl<T> Program<T>
where
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + 'static,
{
    pub fn new(frame: Frame<T>) -> Self {
        Self {
            dimensions: frame.dimensions.y,
            data: RefCell::new(Some(frame)),
        }
    }

    pub fn update_frame(&mut self, frame: Frame<T>) {
        *self.data.borrow_mut() = Some(frame);
    }
}

impl<Message, T> shader::Program<Message> for Program<T>
where
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + 'static,
{
    type State = ();
    type Primitive = Primitive<T>;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive(Mutex::new(match self.data.borrow_mut().take() {
            Some(frame) => State::Pending { frame, bounds },

            _ => State::Prepared {
                bounds,
                image_dimensions: self.dimensions,
            },
        }))
    }
}

#[derive(Debug)]
pub struct Primitive<T>(Mutex<State<T>>);

#[derive(Debug)]
enum State<T> {
    Pending {
        frame: Frame<T>,
        bounds: Rectangle,
    },
    Prepared {
        image_dimensions: Size,
        bounds: Rectangle,
    },
}

impl<T> State<T> {
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
            Self::Pending { frame, .. } => frame.dimensions.y,
        }
    }
}

impl<T> shader::Primitive for Primitive<T>
where
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + 'static,
{
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: Rectangle,
        target_size: iced::Size<u32>,
        _scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        let Ok(mut state) = self.0.lock() else {
            return;
        };

        let target_size = Size {
            width: target_size.width as f32,
            height: target_size.height as f32,
        };

        match state.deref() {
            State::Pending { frame, .. } => {
                if !storage.has::<Pipeline>() {
                    storage.store(Pipeline::new(
                        device,
                        format,
                        frame.dimensions.y,
                        target_size,
                        bounds.size().into(),
                    ));
                }

                let pipeline = storage.get_mut::<Pipeline>().expect("yuv pipeline");

                pipeline.update_uniforms(
                    queue,
                    &Uniforms::new(bounds.size().into(), frame.dimensions.y, target_size),
                );
                pipeline.update_frame(queue, frame);
                pipeline.update_vertices(
                    queue,
                    frame.dimensions.y,
                    bounds.size().into(),
                    target_size,
                )
            }

            State::Prepared {
                image_dimensions, ..
            } => {
                let pipeline = storage.get_mut::<Pipeline>().expect("yuv pipeline");

                pipeline.update_uniforms(
                    queue,
                    &Uniforms::new(bounds.size().into(), *image_dimensions, target_size),
                );
                pipeline.update_vertices(
                    queue,
                    *image_dimensions,
                    bounds.size().into(),
                    target_size,
                );
            }
        }

        *state = State::Prepared {
            bounds: state.bounds(),
            image_dimensions: state.image_dimensions(),
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
