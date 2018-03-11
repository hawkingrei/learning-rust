mod hash;
use hash::{crc16_arr, crc64};
use std::fs::File;
use std::io::prelude::*;

fn main() {
	let mut f = File::create("foo.txt");
	let mut f = match f {
		Ok(file) => file,
		Err(e) => {
			panic!("Failed to open a file: {:?}", e);
		}
	};
	f.write_all(b"Hello, world!");
	let crc64: [u8; 8] = unsafe { std::mem::transmute(crc64(String::from("Hello, world!").as_bytes())) };
	f.write_all(&crc64);
	f.sync_all();
}
