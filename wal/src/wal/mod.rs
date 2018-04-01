mod aligned_buffer;
pub mod env;
mod file_reader_writer;
mod io;
mod log_format;
pub mod log_write;
use hash;
use libc;
use std::mem;
use std::str;
use wal;

pub fn EncodeFixed32(value: u32) -> [u8; 4] {
    if cfg!(target_endian = "little") {
        unsafe { mem::transmute(value.to_le()) }
    } else {
        unsafe { mem::transmute(value.to_be()) }
    }
}

pub fn EncodeFixed64(value: u64) -> [u8; 8] {
    if cfg!(target_endian = "little") {
        unsafe { mem::transmute(value.to_le()) }
    } else {
        unsafe { mem::transmute(value.to_be()) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Code {
    kOk = 0,
    kNotFound = 1,
    kCorruption = 2,
    kNotSupported = 3,
    kInvalidArgument = 4,
    kIOError = 5,
}

#[derive(Debug, Clone)]
struct state {
    state_: Vec<u8>,
}

impl state {
    fn new(code: Code, msg1: String, msg2: String) -> state {
        let msg = msg1 + &String::from(": ") + &msg2;
        let size = mem::size_of_val(&msg);
        let mut state: Vec<u8> = Vec::with_capacity(size + 5);
        state.extend(EncodeFixed32(size as u32).iter().cloned());
        state.extend([code as u8].iter().cloned());
        state.append(&mut msg.into_bytes());
        state { state_: state }
    }

    fn ok() -> state {
        state::new(Code::kOk, "".to_string(), "".to_string())
    }

    fn not_supported() -> state {
        state::new(Code::kNotSupported, "".to_string(), "".to_string())
    }

    fn isOk(&self) -> bool {
        self.state_[4] as u8 == Code::kOk as u8
    }

    fn to_string<'a>(s: &'a state) -> &'a str {
        str::from_utf8(&s.state_[5..]).unwrap()
    }
}

pub trait WritableFile: Sized {
    fn new(filename: String, reopen: bool, preallocation_block_size: usize) -> Self;
    fn append(&mut self, data: Vec<u8>) -> state;
    fn sync(&self) -> state;
    fn close(&self) -> state;
    fn flush(&self) -> state;
    #[cfg(target_os = "linux")]
    fn range_sync(&self, offset: i64, nbytes: i64) -> state;

    fn range_sync(&self, offset: i64, nbytes: i64) -> state {
        return state::ok();
    }

    fn allocate(&self, offset: i64, len: i64) -> state {
        return state::ok();
    }

    fn prepare_write(&mut self, offset: usize, len: usize) {}

    fn positioned_append(data: Vec<u8>, offset: usize) -> state {
        return state::not_supported();
    }

    fn fsync(&self) -> state {
        return self.sync();
    }

    fn get_file_size(&self) -> usize {
        0
    }

    fn use_direct_io(&self) -> bool {
        false
    }
}

#[test]
fn test_state() {
    let s = state::new(Code::kOk, String::from("a"), String::from("b"));
    assert_eq!(true, s.isOk());
    assert_eq!(&String::from("a: b"), state::to_string(&s))
}
