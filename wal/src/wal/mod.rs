pub mod env;
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
struct State {
    state_: Vec<u8>,
}

impl State {
    fn new(code: Code, msg1: String, msg2: String) -> State {
        let msg = msg1 + &String::from(": ") + &msg2;
        let size = mem::size_of_val(&msg);
        let mut state: Vec<u8> = Vec::with_capacity(size + 5);
        state.extend(EncodeFixed32(size as u32).iter().cloned());
        state.extend([code as u8].iter().cloned());
        state.append(&mut msg.into_bytes());
        State { state_: state }
    }

    fn ok() -> State {
        State::new(Code::kOk, "".to_string(), "".to_string())
    }

    fn not_supported() -> State {
        State::new(Code::kNotSupported, "".to_string(), "".to_string())
    }

    fn isOk(s: State) -> bool {
        s.state_[4] as u8 == Code::kOk as u8
    }

    fn to_string<'a>(s: &'a State) -> &'a str {
        str::from_utf8(&s.state_[5..]).unwrap()
    }
}

trait WritableFile {
    fn new(filename: String, reopen: bool, preallocation_block_size: usize) -> Self;
    fn append(&self, data: Vec<u8>) -> Result<State, State>;
    fn sync(&self) -> Result<State, State>;
    fn close(&self) -> Result<State, State>;
    #[cfg(target_os = "linux")]
    fn allocate(&self, offset: i64, len: i64);
    #[cfg(target_os = "linux")]
    fn prepare_write(&mut self, offset: usize, len: usize);

    fn positioned_append(data: Vec<u8>, offset: usize) -> Result<State, State> {
        return Err(State::not_supported());
    }

    fn fsync(&self) -> Result<State, State> {
        return self.sync();
    }
}

#[test]
fn test_state() {
    let s = State::new(Code::kOk, String::from("a"), String::from("b"));
    assert_eq!(true, State::isOk(s.clone()));
    assert_eq!(&String::from("a: b"), State::to_string(&s))
}
