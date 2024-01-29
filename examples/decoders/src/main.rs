use std::fs::File;
use std::io::BufReader;

use iced::{Element, Sandbox, Settings, Size};

use decoders::container::mp4;
use hacky_widget::Video;
use vanilla_iced::yuv;

pub struct Frame(decoders::video::h264::SomeYuv);

impl From<Frame> for yuv::Frame<Vec<u8>> {
    fn from(Frame(data): Frame) -> Self {
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
    Player::run(Settings {
        window: iced::window::Settings {
            size: Size {
                width: 640.0,
                height: 360.0,
            },
            ..Default::default()
        },
        ..Default::default()
    })
}

struct Player {
    mp4: mp4::Container<BufReader<File>>,
}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Sandbox for Player {
    type Message = Message;

    fn new() -> Self {
        let mp4 = mp4::Container::from_file("../../_sample_data/h264.mp4").expect("mp4");

        Self { mp4 }
    }

    fn title(&self) -> String {
        String::from("Iced Iced Baby")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<Message> {
        let stream = self.mp4.h264_stream().expect("stream");

        Video::new(stream).into()
    }
}
