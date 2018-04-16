extern crate gif;
extern crate libc;
use gif::Frame;
use libc::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem;

#[no_mangle]
pub extern "C" fn get_first_frame(
    ptr: *const libc::uint8_t,
    length: libc::size_t,
    rptr: *mut u8,
) -> usize {
    unsafe {
        let mut input: Vec<u8> =
            std::slice::from_raw_parts(ptr as *const u8, length as usize).to_vec();
        let mut decoder = gif::Decoder::new(&*input);
        let mut decoder = decoder.read_info().unwrap();
        let mut image = Vec::from_raw_parts(rptr, 0, length as usize);
        {
            let readimage = &mut image;
            let mut encoder = gif::Encoder::new(
                //&mut image,
                readimage,
                decoder.width(),
                decoder.height(),
                match decoder.global_() {
                    // The division was valid
                    Some(x) => &x,
                    // The division was invalid
                    None => &[],
                },
            ).unwrap();
            match decoder.read_next_frame().unwrap() {
                Some(frame) => encoder.write_frame(&frame).unwrap(),
                None => (),
            };
        }
        let rlen = image.len();
        mem::forget(rptr);
        return rlen;
    }
}
