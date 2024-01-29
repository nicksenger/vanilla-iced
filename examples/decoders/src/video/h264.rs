use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::rc::Rc;

use openh264::decoder::Decoder;
use openh264::formats::YUVSource;
use openh264::Error;

// TODO: figure out if there's some standard upper bound for # unordered frames
const HEAP_SIZE: usize = 8;

#[derive(Clone)]
pub struct Stream<'a>(Rc<RefCell<Inner<'a>>>);

pub struct Inner<'a> {
    decoder: Decoder,
    frames: Box<dyn Iterator<Item = AnnexBFrame> + 'a>,
    yuv_buffer: BinaryHeap<Reverse<DecodedYuv>>,
    j: usize,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
}

pub struct AnnexBFrame {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: i32,
    pub bytes: Vec<u8>,
}

struct DecodedYuv {
    t: i64,
    yuv: SomeYuv,
}

impl Eq for DecodedYuv {}

impl PartialEq for DecodedYuv {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl Ord for DecodedYuv {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.t.cmp(&other.t)
    }
}

impl PartialOrd for DecodedYuv {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Stream<'a> {
    pub fn from_annex_b_packets(
        width: u32,
        height: u32,
        frame_rate: f64,
        frames: impl Iterator<Item = AnnexBFrame> + 'a,
    ) -> Result<Self, Error> {
        let decoder = Decoder::new()?;

        let x = Inner {
            decoder,
            frames: Box::new(frames),
            yuv_buffer: BinaryHeap::with_capacity(HEAP_SIZE),
            j: 0,
            width,
            height,
            frame_rate,
        };

        Ok(Self(Rc::new(RefCell::new(x))))
    }

    pub fn width(&self) -> u32 {
        self.0.borrow().width
    }

    pub fn height(&self) -> u32 {
        self.0.borrow().height
    }

    pub fn frame_rate(&self) -> f64 {
        self.0.borrow().frame_rate
    }

    pub fn next_frame(&self, i: usize) -> Option<SomeYuv> {
        let mut inner = self.0.borrow_mut();

        let skip = i - inner.j;
        if skip > HEAP_SIZE {
            // decoding fell behind, skip until keyframe
            for _ in 0..skip {
                let _ = inner.frames.next();
                inner.j += 1;
            }
        }

        loop {
            'next: while let Some(frame) = inner.frames.next() {
                while inner.yuv_buffer.len() < HEAP_SIZE {
                    let Some(yuv) = inner
                        .decoder
                        .decode(frame.bytes.as_ref())
                        .ok()
                        .flatten()
                        .map(|yuv| SomeYuv {
                            strides: yuv.strides_yuv(),
                            y_dim: yuv.dimension_y(),
                            u_dim: yuv.dimension_u(),
                            v_dim: yuv.dimension_v(),
                            y: yuv.y().to_vec(),
                            u: yuv.u().to_vec(),
                            v: yuv.v().to_vec(),
                        })
                    else {
                        continue 'next;
                    };

                    inner.yuv_buffer.push(Reverse(DecodedYuv {
                        yuv,
                        t: frame.start_time as i64 + frame.rendering_offset as i64,
                    }));
                }

                break;
            }

            let yuv = inner
                .yuv_buffer
                .pop()
                .map(|Reverse(DecodedYuv { yuv, .. })| yuv)?;

            inner.j += 1;

            if inner.j >= i {
                return Some(yuv);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SomeYuv {
    pub strides: (usize, usize, usize),
    pub y_dim: (usize, usize),
    pub u_dim: (usize, usize),
    pub v_dim: (usize, usize),
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}
