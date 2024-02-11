use itertools::Itertools;

#[derive(Debug)]
/// Supported internal types for rendering
pub(crate) enum Renderable {
    Y444 {
        data: Vec<u8>,
        dimensions: Size<u32>,
    },
    I420 {
        data: Vec<u8>,
        dimensions: Size<u32>,
    },
}

impl Renderable {
    pub fn dimensions(&self) -> Size<u32> {
        match self {
            Self::I420 { dimensions, .. } | Self::Y444 { dimensions, .. } => *dimensions,
        }
    }

    pub fn y(&self) -> &[u8] {
        match self {
            Self::I420 { data, .. } => &data[..data.len() / 6 * 4],
            Self::Y444 { data, .. } => &data[..data.len() / 3],
        }
    }

    pub fn u(&self) -> &[u8] {
        match self {
            Self::I420 { data, .. } => &data[data.len() / 6 * 4..data.len() / 6 * 5],
            Self::Y444 { data, .. } => &data[data.len() / 3..data.len() / 3 * 2],
        }
    }

    pub fn v(&self) -> &[u8] {
        match self {
            Self::I420 { data, .. } => &data[data.len() / 6 * 5..],
            Self::Y444 { data, .. } => &data[data.len() / 3 * 2..],
        }
    }

    pub fn downsampling_factor(&self) -> f32 {
        match self {
            Self::I420 { .. } => 2.0,
            Self::Y444 { .. } => 1.0,
        }
    }
}

/// Supported YUV formats
#[derive(Clone, Copy, Debug)]
pub enum Format {
    I420,
    Y444,
    NV12,
}

#[derive(Debug, Clone)]
pub struct Yuv {
    pub format: Format,
    pub data: Vec<u8>,
    pub dimensions: Size<u32>,
}

impl From<Yuv> for Renderable {
    fn from(
        Yuv {
            mut data,
            dimensions,
            format,
        }: Yuv,
    ) -> Self {
        match format {
            Format::I420 => Renderable::I420 { data, dimensions },

            Format::Y444 => Renderable::Y444 { data, dimensions },

            Format::NV12 => {
                let n = data.len() / 6;
                let chroma = data.split_off(n * 4);
                let (u, v): (Vec<_>, Vec<_>) =
                    chroma.into_iter().enumerate().partition_map(|(i, b)| {
                        if i % 2 == 0 {
                            itertools::Either::Left(b)
                        } else {
                            itertools::Either::Right(b)
                        }
                    });

                data.extend(u);
                data.extend(v);

                Renderable::I420 { data, dimensions }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

impl std::ops::Mul<f32> for Size<f32> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl From<Size<u32>> for Size<f32> {
    fn from(size: Size<u32>) -> Self {
        (size.width as f32, size.height as f32).into()
    }
}

impl From<(f32, f32)> for Size {
    fn from(value: (f32, f32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<(u32, u32)> for Size<u32> {
    fn from(value: (u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<iced::Size<f32>> for Size {
    fn from(size: iced::Size<f32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}
