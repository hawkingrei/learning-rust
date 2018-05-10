extern crate gif;
extern crate libc;
use std::io;
use std::mem;

#[inline]
fn round_up(x: usize, y: usize) -> usize {
    return ((x + y - 1) / y) * y;
}

#[no_mangle]
pub extern "C" fn get_first_frame(
    ptr: *const libc::uint8_t,
    length: libc::size_t,
    width: *mut u16,
    height: *mut u16,
    rptr: *mut u8,
) -> usize {
    let mut rlen;
    let mut image;
    let mut image2;
    let input: Vec<u8>;
    unsafe {
        image = Vec::from_raw_parts(rptr, 0, round_up(length as usize, 64));
        image2 = image.clone();
        input = Vec::from_raw_parts(ptr as *mut u8, length as usize, length as usize);
    }
    let mut is_error = false;

    unsafe {
        {
            let mut cursor = io::Cursor::new(&input);
            let mut decoder = gif::Decoder::new(cursor);
            let mut decode;
            match decoder.read_info() {
                Ok(x) => {
                    decode = x;
                    let mut readimage = &mut image2;
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
                    match decode.read_next_frame() {
                        Ok(r) => match r {
                            Some(ref frame) => encoder.write_frame(frame).unwrap(),
                            None => (),
                        },
                        Err(e) => {
                            println!("read_next_frame happen error {:?}", e);
                            is_error = true;
                        }
                    };
                }
                Err(_) => {}
            }
        }
        if is_error {
            rlen = 0;
            image.set_len(rlen);
            image.copy_from_slice(&[]);
        } else {
            rlen = image2.len();
            image.set_len(rlen);
            image.copy_from_slice(&image2.as_slice());
        }
        mem::forget(image);
        mem::forget(rptr);
        mem::forget(input);
        return rlen;
    }
}
