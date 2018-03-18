mod hash;
mod wal;
extern crate zstd;
#[cfg(feature = "zstd")]
use zstd::block::compress;
use hash::{crc16_arr, crc64};
use std::fs::File;
use wal::log_write;
use std::io::prelude::*;
use std::mem::size_of_val;
use std::io::SeekFrom;

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
	let crc64data: [u8; 8] = unsafe { std::mem::transmute(crc64(inputcompress)) };
	f.write_all(&crc64data);
	f.sync_all();

	let mut ff = match File::open("foo.txt") {
		Ok(file) => file,
		Err(e) => {
			panic!("Failed to open a file: {:?}", e);
		}
	};
	let mut datasize = [0; 8];
	ff.read(&mut datasize);
	let realsize = unsafe { std::mem::transmute::<[u8; 8], usize>(datasize) };
	f.seek(SeekFrom::Current(8));

	let mut data = vec![0; realsize];
	ff.read(&mut data);
	f.seek(SeekFrom::Current(realsize as i64));

	let mut crcdata = [0; 8];
	ff.read(&mut crcdata);
	let crc_expected: u64 = unsafe { std::mem::transmute(crcdata) };

	let crc_actual = crc64(&data);
	if crc_expected == crc_actual {
		println!("ok");
	};
}
