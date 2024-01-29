use std::time::Duration;

use iced::advanced::layout;
use iced::advanced::mouse::Cursor;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::event::{self, Event};
use iced::widget::{Component, Shader, Space};
use iced::window::RedrawRequest;
use iced::{window, Size};
use iced::{Element, Length, Rectangle};
use vanilla_iced::yuv;
use web_time::Instant;

mod types;

pub use types::VideoStream;

const PLAYBACK_RATE: f32 = 1.0;

struct Inner<'a, T> {
    program: Option<&'a yuv::Program<T>>,
}

impl<'a, T, Message> Component<Message> for Inner<'a, T>
where
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + 'static,
{
    type State = ();
    type Event = ();

    fn update(&mut self, _state: &mut Self::State, _event: Self::Event) -> Option<Message> {
        None
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event> {
        if let Some(program) = self.program {
            Shader::new(program).into()
        } else {
            Space::new(0, 0).into()
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct Video<'a, T> {
    width: Length,
    height: Length,
    frame_duration: Duration,
    content: Box<dyn VideoStream<T> + 'a>,
    video_width: u32,
    video_height: u32,
}

impl<'a, T> Video<'a, T>
where
    T: AsRef<[u8]>,
{
    pub fn new(content: impl VideoStream<T> + 'a) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            frame_duration: Duration::from_secs_f64(1.0 / content.frame_rate()),
            video_width: content.width(),
            video_height: content.height(),
            content: Box::new(content),
        }
    }

    /// Sets the width of the [`Video`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Video`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Video<'a, T>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::pipeline::Renderer,
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + Default + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<T>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<T>::new(
            self.video_width as f32,
            self.video_height as f32,
        ))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let shader: Element<'_, Message, Theme, Renderer> = {
            let state = tree.state.downcast_ref::<State<T>>();
            Element::from(
                Shader::<Message, &yuv::Program<T>>::new(&state.program)
                    .width(self.width)
                    .height(self.height),
            )
        };
        shader
            .as_widget()
            .layout(&mut Tree::new(&shader), renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor_position: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State<T>>();

        let mut progress_frame = |i| {
            if let Some(frame) = self.content.next(i) {
                state.program.update_frame(frame);
            }
        };

        if let (Some(&first), Some(&last)) = (state.first_draw.as_ref(), state.last_draw.as_ref()) {
            if let Event::Window(_id, window::Event::RedrawRequested(now)) = event {
                    if now.saturating_duration_since(last) >= self.frame_duration {
                        state.last_draw = Some(now);
                        progress_frame(
                            ((now - first).as_secs_f32()
                                / (self.frame_duration.as_secs_f32() / PLAYBACK_RATE))
                                as usize,
                        );
                    }
                    shell.request_redraw(RedrawRequest::At(now + self.frame_duration));
            }
        } else {
            let now = Instant::now();
            state.first_draw = Some(now);
            state.last_draw = Some(now);
            progress_frame(0);
            shell.request_redraw(RedrawRequest::At(now + self.frame_duration))
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<T>>();

        let shader = Element::from(Shader::<Message, &yuv::Program<T>>::new(&state.program));
        shader.as_widget().draw(
            &Tree::new(&shader),
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }
}

impl<'a, T, Message, Theme, Renderer> From<Video<'a, T>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::pipeline::Renderer + 'a,
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + Default + 'static,
{
    fn from(video: Video<'a, T>) -> Self {
        Self::new(video)
    }
}

struct State<T> {
    program: yuv::Program<T>,
    last_draw: Option<Instant>,
    first_draw: Option<Instant>,
}

impl<T> State<T>
where
    T: AsRef<[u8]> + std::fmt::Debug + Send + Sync + Default + 'static,
{
    fn new(image_width: f32, image_height: f32) -> Self {
        Self {
            program: yuv::Program::new(yuv::Frame {
                strides: yuv::Strides { y: 0, u: 0, v: 0 },
                dimensions: yuv::Dimensions {
                    y: (image_width, image_height).into(),
                    u: (image_width / 2.0, image_height / 2.0).into(),
                    v: (image_width / 2.0, image_height / 2.0).into(),
                },
                y: Default::default(),
                u: Default::default(),
                v: Default::default(),
            }),
            last_draw: None,
            first_draw: None,
        }
    }
}
