pub mod audio;
pub mod container;
pub mod video;

#[cfg(feature = "bin")]
mod widget {
    use hacky_widget::VideoStream;
    use vanilla_iced::{Format, Size, Yuv};

    impl<'a> VideoStream for crate::video::h264::Stream<'a> {
        fn width(&self) -> u32 {
            self.width()
        }

        fn height(&self) -> u32 {
            self.height()
        }

        fn frame_rate(&self) -> f64 {
            self.frame_rate()
        }

        fn next(&mut self, i: usize) -> Option<Yuv> {
            self.next_frame(i).map(Into::into)
        }
    }

    impl From<crate::video::h264::SomeYuv> for Yuv {
        fn from(data: crate::video::h264::SomeYuv) -> Self {
            let mut bytes = data.y;
            bytes.extend(data.u);
            bytes.extend(data.v);

            Self {
                format: Format::I420,
                dimensions: Size {
                    width: data.y_dim.0 as f32,
                    height: data.y_dim.1 as f32,
                },
                data: bytes,
            }
        }
    }
}
