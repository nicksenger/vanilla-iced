# Vanilla Iced

![iced_baby_vip](./iced_baby_vip.jpg)

Custom shader for rendering [YUV](https://en.wikipedia.org/wiki/Y%E2%80%B2UV) / [YCbCr](https://en.wikipedia.org/wiki/YCbCr) data using [Iced](https://github.com/iced-rs/iced.git). 

Check out the [examples](./examples) to see it in action!


### Usage

```rust
fn view(&self) -> Element<Message> {
    // Get some YUV
    let yuv = vanilla_iced::Yuv {
        format: Format::I420, // yuv format
        data: vec![..] // raw yuv data
    };

    // Render it
    iced::widget::shader(vanilla_iced::Program::new(yuv)).into()
}

fn update(&mut self, message: Message) -> iced::Command<Message> {
    match message {
        Message::GotSomeNewYuv(frame) => {
            // Update a shader program in your app state with new YUV data
            self.program.update_frame(frame);
        }
    }
}
```

### Goals

- [x] Render I420
- [x] Render Y444
- [ ] Support other common 4:2:0 YUV formats by conversion to I420
    - [x] NV12
- [ ] Support other common 4:4:4 YUV formats by conversion to Y444

### Motivations

Typically the output from decoding compressed video formats is YUV frames. While YUV can be converted to RGBA on the CPU for rendering with Iced's `Image` widget, doing so is not particularly efficient. This project seeks to move YUV conversion to the GPU using a dedicated shader, supporting the development of video players and other multimedia applications with Iced.
