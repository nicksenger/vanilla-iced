pub trait VideoStream {
    /// Format of each video frame
    fn format(&self) -> vanilla_iced::Format;

    /// Size of the video
    fn dimensions(&self) -> vanilla_iced::Size<u32>;

    /// Frame rate in fps
    fn frame_rate(&self) -> f64;

    /// Returns the YUV data for the requested frame,
    /// or None if the stream is exhausted
    fn next(&mut self, i: usize) -> Option<vanilla_iced::Yuv>;
}
