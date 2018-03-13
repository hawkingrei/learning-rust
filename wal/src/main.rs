mod hash;
extern crate zstd;
#[cfg(feature = "zstd")]
use zstd::block::compress;
use hash::{crc16_arr, crc64};
use std::fs::File;
use std::io::prelude::*;
use std::mem::size_of_val;

fn main() {
	let mut f = match File::create("foo.txt") {
		Ok(file) => file,
		Err(e) => {
			panic!("Failed to open a file: {:?}", e);
		}
	};
	let input = b"aaaaaaaaaaaa";
	let inputcompress = &zstd::block::compress(input, 5).unwrap();
	let size = unsafe { std::mem::transmute::<usize, [u8; 8]>(size_of_val(inputcompress)) };
	f.write_all(&size);
	f.write_all(inputcompress);
	let crc64: [u8; 8] = unsafe { std::mem::transmute(crc64(inputcompress)) };
	f.write_all(&crc64);
	f.sync_all();

	let mut ff = match File::open("foo.txt") {
		Ok(file) => file,
		Err(e) => {
			panic!("Failed to open a file: {:?}", e);
		}
	};
	let mut buffer = Vec::new();
	ff.read_to_end(&mut buffer);
	let ssize = unsafe { std::mem::transmute::<[u8; 8], u64>(buffer[0..8]) };
	let data = &buffer[8..ssize + 8];
	let crc_expected: u64 = unsafe { std::mem::transmute(buffer[ssize + 8..ssize + 8 + 8]) };
	let crc_actual = crc64(data);
	if crc_expected != crc_actual {
		println!("ok");
	};
}
