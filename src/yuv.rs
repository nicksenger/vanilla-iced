#[derive(Debug)]
/// Supported internal types for rendering
pub(crate) enum Renderable {
    Y444 { data: Vec<u8>, dimensions: Size },
    I420 { data: Vec<u8>, dimensions: Size },
}

impl Renderable {
    pub fn dimensions(&self) -> Size {
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

    pub fn sampling_factor(&self) -> f32 {
        match self {
            Self::I420 { .. } => 2.0,
            Self::Y444 { .. } => 1.0
        }
    }
}

/// Supported YUV formats
#[derive(Clone, Copy, Debug)]
pub enum Format {
    I420,
    Y444,
}

#[derive(Debug, Clone)]
pub struct Yuv {
    pub format: Format,
    pub data: Vec<u8>,
    pub dimensions: Size,
}

impl From<Yuv> for Renderable {
    fn from(yuv: Yuv) -> Self {
        match yuv.format {
            Format::I420 => Renderable::I420 {
                data: yuv.data,
                dimensions: yuv.dimensions,
            },
            Format::Y444 => Renderable::Y444 {
                data: yuv.data,
                dimensions: yuv.dimensions,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl std::ops::Mul<f32> for Size {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            width: self.width * rhs,
            height: self.height * rhs,
        }
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

impl From<iced::Size<f32>> for Size {
    fn from(size: iced::Size<f32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}
