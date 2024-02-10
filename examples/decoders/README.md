This example demonstrates the use of Vanilla Iced in combination with a custom demuxing/decoding pipeline built with [mp4-rust](https://github.com/alfg/mp4-rust) and [openh264-rs](https://github.com/ralfbiedert/openh264-rs), as well as a [hacky widget](../hacky_widget/README.md), to render video in Iced.

To run the example use `cargo run --features bin` from this directory.

It should compile out-of-the box on most platforms except WASM, but if not see [openh264-rs](https://github.com/ralfbiedert/openh264-rs) since its underlying C library is the most likely issue.

Please note that the segment from Vanilla Ice's hit 1990 single "Ice Ice Baby" is provided for educational purposes only, in accordance with the [copyright disclaimer](../../_sample_data/COPYRIGHT_DISCLAIMER.md).