use std::time::Duration;

use iced::advanced::layout;
use iced::advanced::mouse::Cursor;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::event::{self, Event};
use iced::widget::Shader;
use iced::window::RedrawRequest;
use iced::{window, Size};
use iced::{Element, Length, Rectangle};
use web_time::Instant;

use vanilla_iced::{Format, Program, Yuv};

mod types;

pub use types::VideoStream;

const PLAYBACK_RATE: f32 = 1.0;

pub struct Video<'a> {
    width: Length,
    height: Length,
    frame_duration: Duration,
    content: Box<dyn VideoStream + 'a>,
    video_width: u32,
    video_height: u32,
}

impl<'a> Video<'a> {
    pub fn new(content: impl VideoStream + 'a) -> Self {
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

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Video<'a>
where
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::pipeline::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(
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
            let state = tree.state.downcast_ref::<State>();
            Element::from(
                Shader::<Message, &Program>::new(&state.program)
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
        let state = tree.state.downcast_mut::<State>();

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
        let state = tree.state.downcast_ref::<State>();

        let shader = Element::from(Shader::<Message, &Program>::new(&state.program));
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

impl<'a, Message, Theme, Renderer> From<Video<'a>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced_wgpu::primitive::pipeline::Renderer + 'a,
{
    fn from(video: Video<'a>) -> Self {
        Self::new(video)
    }
}

struct State {
    program: Program,
    last_draw: Option<Instant>,
    first_draw: Option<Instant>,
}

impl State {
    fn new(image_width: f32, image_height: f32) -> Self {
        Self {
            program: Program::new(Yuv {
                format: Format::Y444,
                data: vec![],
                dimensions: (image_width, image_height).into(),
            }),
            last_draw: None,
            first_draw: None,
        }
    }
}
