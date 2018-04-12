#![crate_type = "staticlib"]
extern crate gif;
extern crate libc;
use gif::Frame;
use gif::SetParameter;
use std::ffi::CStr;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn get_first_frame(image: *const libc::c_char) -> CString {
    let mut input: Vec<u8> = unsafe { CStr::from_ptr(image).to_bytes().iter().cloned().collect() };
    let mut decoder = gif::Decoder::new(&*input);
    let mut decoder = decoder.read_info().unwrap();
    let mut image = Vec::new();
    {
        let readimage = &mut image;
        let mut encoder = gif::Encoder::new(
            //&mut image,
            readimage,
            decoder.width(),
            decoder.height(),
            match decoder.global_palette() {
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
    unsafe {
        return CString::from_vec_unchecked(image);
    }
}
