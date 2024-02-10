pub trait VideoStream {
    /// Width of each frame in px
    fn width(&self) -> u32;

    /// Height of each frame in px
    fn height(&self) -> u32;

    /// Frame rate in fps
    fn frame_rate(&self) -> f64;

    /// Returns the YUV data for the requested frame,
    /// or None if the stream is exhausted
    fn next(&mut self, i: usize) -> Option<vanilla_iced::Yuv>;
}
