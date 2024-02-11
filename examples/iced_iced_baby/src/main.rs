use iced::{Element, Sandbox, Settings, Size};

use hacky_widget::{Video, VideoStream};

mod player;

use player::Player;

pub fn main() -> iced::Result {
    App::run(Settings {
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

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<Message> {
        let mut player = Player::new().expect("player");
        while player.next(0).is_none() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        Video::new(player).into()
    }
}
