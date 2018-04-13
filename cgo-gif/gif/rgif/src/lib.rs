extern crate gif;
extern crate libc;
use gif::Frame;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem;

#[repr(C)]
pub struct Image {
    pub data: *mut u8,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn get_first_frame(ptr: *const libc::uint8_t, length: libc::size_t) -> Image {
    unsafe {
        let mut input: Vec<u8> =
            std::slice::from_raw_parts(ptr as *const u8, length as usize).to_vec();
        println!("rust len {}", input.len());
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
        let result = Image {
            data: image.as_mut_ptr(),
            len: image.len(),
        };
        mem::forget(image);
        return result;
    }
}
