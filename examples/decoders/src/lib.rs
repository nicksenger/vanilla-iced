pub mod audio;
pub mod container;
pub mod video;

#[cfg(feature = "bin")]
mod widget {
    use hacky_widget::VideoStream;
    use vanilla_iced::yuv;

    impl<'a> VideoStream<Vec<u8>> for crate::video::h264::Stream<'a> {
        fn width(&self) -> u32 {
            self.width()
        }

        fn height(&self) -> u32 {
            self.height()
        }

        fn frame_rate(&self) -> f64 {
            self.frame_rate()
        }

        fn next(&mut self, i: usize) -> Option<yuv::Frame<Vec<u8>>> {
            self.next_frame(i).map(Into::into)
        }
    }

    impl From<crate::video::h264::SomeYuv> for yuv::Frame<Vec<u8>> {
        fn from(data: crate::video::h264::SomeYuv) -> Self {
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
}
