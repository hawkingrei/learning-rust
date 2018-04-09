extern crate gif;
use gif::Frame;
use gif::SetParameter;
use std::fs::File;
fn main() {
	let mut decoder = gif::Decoder::new(File::open("test.gif").unwrap());
	decoder.set(gif::ColorOutput::RGBA);
	let mut decoder = decoder.read_info().unwrap();
	let mut nframe = Frame::default();
	let mut image = File::create("test1.gif").unwrap();
	let mut encoder = gif::Encoder::new(
		&mut image,
		decoder.width(),
		decoder.height(),
		match decoder.global_palette() {
			// The division was valid
			Some(x) => &x,
			// The division was invalid
			None => &[],
		},
	).unwrap();
	let frame = match decoder.read_next_frame().unwrap() {
		Some(frame) => frame,
		None => std::process::exit(0),
	};
	let mut nframe = Frame::default();
}
