use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use gstreamer::prelude::*;
use num_traits::cast::ToPrimitive;

use vanilla_iced::{Format, Size, Yuv};

const FORMAT: Format = Format::NV12;

fn gstreamer_format_code(format: Format) -> &'static str {
    match format {
        Format::I420 => "I420",
        Format::NV12 => "NV12",
        Format::Y444 => "Y444",
    }
}

#[derive(Clone, Debug, Default)]
pub struct Bytes(Arc<Vec<u8>>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub struct Player {
    source: gstreamer::Bin,
    width: u32,
    height: u32,
    framerate: f64,
    frame: Arc<Mutex<Option<Yuv>>>,
}

impl Player {
    pub fn new() -> Result<Self> {
        // Mostly copied from https://github.com/jazzfool/iced_video_player/blob/master/src/lib.rs
        gstreamer::init()?;

        let mut path = std::env::current_dir()
            .expect("dir")
            .parent()
            .expect("parent")
            .parent()
            .expect("parent")
            .to_path_buf();
        path.push("_sample_data/av1.mp4");

        let source = gstreamer::parse_launch(
            &format!(
                "playbin uri=\"file:///{}\" video-sink=\"videoconvert ! appsink name=app_sink caps=video/x-raw,format={}\"",
                path.to_str().expect("path").replace('\\', "/"),
                gstreamer_format_code(FORMAT)
            )
        )?;
        let source = source
            .downcast::<gstreamer::Bin>()
            .map_err(|_| anyhow!("downcast bin"))?;

        let video_sink: gstreamer::Element = source.property("video-sink").expect("sink").get()?;
        let pad = video_sink.pads().first().cloned().expect("pads");
        let pad = pad
            .dynamic_cast::<gstreamer::GhostPad>()
            .expect("ghost pad");
        let bin = pad
            .parent_element()
            .unwrap()
            .downcast::<gstreamer::Bin>()
            .unwrap();

        let app_sink = bin.by_name("app_sink").unwrap();
        let app_sink = app_sink.downcast::<gstreamer_app::AppSink>().unwrap();

        let frame = Arc::new(Mutex::new(None));
        let frame_ref = Arc::clone(&frame);

        app_sink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gstreamer::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or(gstreamer::FlowError::Error)?;
                    let map = buffer
                        .map_readable()
                        .map_err(|_| gstreamer::FlowError::Error)?;

                    let pad = sink.static_pad("sink").ok_or(gstreamer::FlowError::Error)?;

                    let caps = pad.current_caps().ok_or(gstreamer::FlowError::Error)?;
                    let _s = caps.structure(0).ok_or(gstreamer::FlowError::Error)?;

                    let data = map.as_slice();

                    *frame_ref.lock().map_err(|_| gstreamer::FlowError::Error)? = Some(Yuv {
                        format: FORMAT,
                        // TODO: get this from gstreamer
                        dimensions: Size {
                            width: 1280,
                            height: 720,
                        },
                        data: data.to_vec(),
                    });

                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        source.set_state(gstreamer::State::Playing)?;

        source.state(gstreamer::ClockTime::from_seconds(5)).0?;

        let caps = pad.current_caps().ok_or(anyhow!("caps"))?;
        let s = caps.structure(0).ok_or(anyhow!("structure"))?;
        let framerate = s.get::<gstreamer::Fraction>("framerate")?;
        let width = s.get::<i32>("width")?;
        let height = s.get::<i32>("height")?;

        Ok(Player {
            source,
            width: width as u32,
            height: height as u32,
            framerate: num_rational::Rational32::new(
                *framerate.numer() as _,
                *framerate.denom() as _,
            )
            .to_f64()
            .expect("framerate"),

            frame,
        })
    }

    fn frame(&self) -> Option<Yuv> {
        self.frame.lock().expect("lock").clone()
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.source.set_state(gstreamer::State::Null).unwrap();
    }
}

impl hacky_widget::VideoStream for Player {
    fn format(&self) -> vanilla_iced::Format {
        FORMAT
    }

    fn dimensions(&self) -> vanilla_iced::Size<u32> {
        (self.width, self.height).into()
    }

    fn frame_rate(&self) -> f64 {
        self.framerate
    }

    fn next(&mut self, _i: usize) -> Option<Yuv> {
        self.frame()
    }
}
