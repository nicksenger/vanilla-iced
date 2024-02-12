use iced::{
    theme,
    widget::{container, shader},
    Element, Length, Sandbox, Settings,
};
use serde::{Deserialize, Serialize};

use vanilla_iced::{Format, Program, Size, Yuv};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyYuv {
    pub strides: (usize, usize, usize),
    pub y_dim: (usize, usize),
    pub u_dim: (usize, usize),
    pub v_dim: (usize, usize),
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}

impl From<MyYuv> for Yuv {
    fn from(data: MyYuv) -> Self {
        let mut bytes = data.y;
        bytes.extend(data.u);
        bytes.extend(data.v);

        Self {
            format: Format::I420,
            dimensions: Size {
                width: data.y_dim.0 as u32,
                height: data.y_dim.1 as u32,
            },
            data: bytes,
        }
    }
}

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("Iced Iced Baby")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<Message> {
        let mut yuv: MyYuv =
            bincode::deserialize(include_bytes!("../../../_sample_data/frame.raw")).expect("frame");

        add_d(&mut yuv);

        container(
            shader(Program::new(yuv.into()))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(theme::Container::Box)
        .into()
    }
}

/// Adds green letter "d" to the yuv
fn add_d(yuv: &mut MyYuv) {
    let stride = 672;
    for i in 0..(stride / 24) {
        for line in (30..32).chain(45..47) {
            yuv.u[stride * line + 265 + i] = 0;
            yuv.v[stride * line + 265 + i] = 0;
        }

        for line in 30..46 {
            for offset in 265..267 {
                yuv.u[stride * line + offset] = 0;
                yuv.v[stride * line + offset] = 0;
            }
        }

        for line in 10..46 {
            for offset in 286..289 {
                yuv.u[stride * line + offset] = 0;
                yuv.v[stride * line + offset] = 0;
            }
        }
    }
}
