extern crate gif;
extern crate libc;
use std::mem;

#[no_mangle]
pub extern "C" fn get_first_frame(
    ptr: *const libc::uint8_t,
    length: libc::size_t,
    width: *mut u16,
    height: *mut u16,
    rptr: *mut u8,
) -> usize {
    unsafe {
        let input: Vec<u8> = std::slice::from_raw_parts(ptr as *const u8, length as usize).to_vec();
        let mut decoder = gif::Decoder::new(&*input);
        let mut decode;
        match decoder.read_info() {
            Ok(x) => {
                decode = x;
            }
            Err(_) => return 0,
        }
        let mut image = Vec::from_raw_parts(rptr, 0, length as usize);
        {
            let readimage = &mut image;
            *width = decode.width();
            *height = decode.height();
            let mut encoder = gif::Encoder::new(
                //&mut image,
                readimage,
                *width,
                *height,
                match decode.global_palette() {
                    // The division was valid
                    Some(x) => &x,
                    // The division was invalid
                    None => &[],
                },
            ).unwrap();
            match decode.read_next_frame().unwrap() {
                Some(frame) => encoder.write_frame(&frame).unwrap(),
                None => (),
            };
        }
        let rlen = image.len();
        mem::forget(rptr);
        return rlen;
    }
}
