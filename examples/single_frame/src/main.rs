use iced::{widget::shader, Element, Length, Sandbox, Settings};

use serde::{Deserialize, Serialize};
use vanilla_iced::yuv;

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

impl From<MyYuv> for yuv::Frame<Vec<u8>> {
    fn from(data: MyYuv) -> Self {
        Self {
            strides: yuv::Strides {
                y: data.strides.0,
                u: data.strides.1,
                v: data.strides.2,
            },
            dimensions: yuv::Dimensions {
                y: yuv::Size {
                    width: data.y_dim.0 as f32,
                    height: data.y_dim.1 as f32,
                },
                u: yuv::Size {
                    width: data.u_dim.0 as f32,
                    height: data.u_dim.1 as f32,
                },
                v: yuv::Size {
                    width: data.v_dim.0 as f32,
                    height: data.v_dim.1 as f32,
                },
            },
            y: data.y,
            u: data.u,
            v: data.v,
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

        shader(yuv::Program::new(yuv.into()))
            .width(Length::Fill)
            .height(Length::Fill)
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
