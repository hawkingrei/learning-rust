pub mod log_write;
mod log_format;
use std::mem;
use hash;

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

#[derive(Debug)]
struct State {
    state_: Vec<u8>,
}

impl State {
    fn new(code: Code, msg1: String, msg2: String) -> State {
        let msg = msg1 + &String::from(": ") + &msg2;
        let mut state: Vec<u8> = Vec::with_capacity(msg.len() + 5);
        state.clone_from_slice(unsafe { mem::transmute::<usize, &[u8; 3]>(msg.len()) });
        state[4] = unsafe { mem::transmute(code as u8) };
        state[5..].clone_from_slice(&msg.into_bytes());
        State { state_: state }
    }
}

fn isOk(s: State) -> bool {
    s.state_[4] as u8 == Code::kOk as u8
}

#[test]
fn test_crc64() {
    let s = State::new(Code::kOk, String::from("a"), String::from("b"));
    assert_eq!(true, isOk(s));
}
