extern crate cv;
extern crate gif;
extern crate num_iter;
extern crate num_rational;
extern crate num_traits;
extern crate resize;

use cv::core::*;
use cv::imgproc::*;
use cv::*;
use gif::Frame;
use gif::SetParameter;
use resize::Pixel::RGB24;
use resize::Type::Lanczos3;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::Write;
use std::time::SystemTime;

#[inline]
fn copy_memory(src: &[u8], mut dst: &mut [u8]) {
    let len_src = src.len();
    assert!(dst.len() >= len_src);
    dst.write_all(src).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let metadata = match fs::metadata(&args[1]) {
        Ok(x) => x.len(),
        Err(_) => 0,
    };
    let mut f = File::open(&args[1]).unwrap();

    let mut input = Vec::new();
    f.read_to_end(&mut input).unwrap();
    let mut decoder = gif::Decoder::new(&*input);
    decoder.set(gif::ColorOutput::RGBA);
    let sys_time = SystemTime::now();
    let mut decoder = decoder.read_info().unwrap();
    let nframe = Frame::default();
    let mut image = Vec::new();

    //println!("decoder.global_palette() {:?}", decoder.global_palette());
    //File::create(&args[1].replace(".gif", "_1.gif")).unwrap();
    {
        let readimage = &mut image;
        let width = decoder.width();
        let height = decoder.height();
        println!("{}", decoder.width());
        println!("{}", decoder.height());
        let mut encoder = gif::Encoder::new(
            //&mut image,
            readimage,
            width,
            height,
            match decoder.global_palette() {
                // The division was valid
                Some(x) => &x,
                // The division was invalid
                None => &[],
            },
        ).unwrap();
        match decoder.read_next_frame().unwrap() {
            Some(frame) => {
                let mut newframe = Frame::default();
                newframe.width = width / 2;
                newframe.height = height / 2;

                {
                    let src = &frame.buffer;
                    let mut dst = vec![0; width as usize * height as usize / 4];
                    resize::resize(
                        width as usize,
                        height as usize,
                        frame.width as usize,
                        frame.height as usize,
                        RGB24,
                        Lanczos3,
                        &src,
                        &mut dst,
                    );
                    newframe = gif::Frame::from_rgb(width / 2, height / 2, &mut *dst);
                }

                encoder.write_frame(&newframe).unwrap();
            }
            None => std::process::exit(0),
        }
    }

    println!("img {:?}", image.len());
    let sys_next_time = SystemTime::now();
    let difference = sys_next_time
        .duration_since(sys_time)
        .expect("SystemTime::duration_since failed");
    println!("{} {} {:?}", args[1], metadata, difference);
    let mut f = File::create("test1.gif").expect("Unable to create file");
    for i in image {
        f.write_all((&[i])).expect("Unable to write data");
    }
}
