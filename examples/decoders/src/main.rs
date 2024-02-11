use std::fs::File;
use std::io::BufReader;

use iced::{Element, Sandbox, Settings, Size};

use decoders::container::mp4;
use hacky_widget::Video;

pub fn main() -> iced::Result {
    Player::run(Settings {
        window: iced::window::Settings {
            size: Size {
                width: 1280.0,
                height: 720.0,
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
