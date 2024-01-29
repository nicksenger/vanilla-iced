use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};

use mp4::Mp4Reader;
use thiserror::Error;

use crate::video::h264::AnnexBFrame;

mod bitstream_converter;

#[derive(Error, Debug)]
pub enum Error {
    #[error("bitstream conversion error: {0:?}")]
    BitstreamConversion(#[from] bitstream_converter::Error),
    #[error("{0:?}")]
    Mp4Error(#[from] mp4::Error),
    #[error("no matching stream found")]
    StreamNotFound,
    #[error("io error: {0:?}")]
    IoError(#[from] std::io::Error),

    #[error("h.264 error: {0:?}")]
    OpenH264Error(#[from] openh264::Error),
}

pub struct Container<T>(RefCell<mp4::Mp4Reader<T>>);

impl Container<BufReader<File>> {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let size = file.metadata()?.len();
        let reader = BufReader::new(file);

        let mp4 = Mp4Reader::read_header(reader, size)?;

        Ok(Self(RefCell::new(mp4)))
    }
}

impl<'a> Container<Cursor<&'a [u8]>> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        let mp4 = mp4::Mp4Reader::read_header(Cursor::new(bytes), bytes.len() as u64)?;

        Ok(Self(RefCell::new(mp4)))
    }
}

impl<T: Read + Seek> Container<T> {
    pub fn h264_stream(&self) -> Result<crate::video::h264::Stream, Error> {
        let (mut bitstream_converter, track_id, sample_count, width, height, frame_rate) = {
            let inner = self.0.borrow_mut();
            let track = inner
                .tracks()
                .iter()
                .find(|(_, t)| t.media_type().ok() == Some(mp4::MediaType::H264))
                .ok_or(Error::StreamNotFound)?
                .1;
            let track_id = track.track_id();
            (
                bitstream_converter::Mp4BitstreamConverter::for_mp4_track(track)?,
                track_id,
                track.sample_count(),
                track.width(),
                track.height(),
                track.frame_rate(),
            )
        };

        let annex_b_frames = (1..sample_count + 1).filter_map(move |i| {
            self.0
                .borrow_mut()
                .read_sample(track_id, i)
                .ok()
                .flatten()
                .map(|sample| {
                    let mut bytes = vec![];
                    bitstream_converter.convert_packet(&sample.bytes, &mut bytes);

                    crate::video::h264::AnnexBFrame {
                        start_time: sample.start_time,
                        duration: sample.duration,
                        rendering_offset: sample.rendering_offset,
                        bytes,
                    }
                })
        });

        crate::video::h264::Stream::from_annex_b_packets(
            width as u32,
            height as u32,
            frame_rate,
            Box::new(annex_b_frames) as Box<dyn Iterator<Item = AnnexBFrame>>,
        )
        .map_err(Error::OpenH264Error)
    }
}
