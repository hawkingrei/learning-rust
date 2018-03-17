mod hash;
extern crate zstd;
#[cfg(feature = "zstd")]
use zstd::block::compress;
use hash::{crc16_arr, crc64};
use std::fs::File;
use std::io::prelude::*;
use std::mem::size_of_val;

enum RecordType {
  kZeroType = 0,
  kFullType = 1,

  // For fragments
  kFirstType = 2,
  kMiddleType = 3,
  kLastType = 4,

  // For recycled log files
  kRecyclableFullType = 5,
  kRecyclableFirstType = 6,
  kRecyclableMiddleType = 7,
  kRecyclableLastType = 8,
}

fn main() {
    let mut f = File::create("foo.txt");
    let mut f = match f {
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
}
