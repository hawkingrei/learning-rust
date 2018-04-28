extern crate cv;
extern crate gif;

use cv::*;
use cv::core::*;
use cv::imgproc::*;
use gif::Frame;
use gif::SetParameter;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::Write;
use std::time::SystemTime;

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
                
                let mat = Mat::from_buffer(height as i32, width as i32, CvType::Cv8UC3 as i32, &frame.buffer.clone().into_owned().to_vec());
                mat.resize_to( Size2i::new(300,169),InterpolationFlag::InterNearst);
                println!("{:?}", frame.buffer.len());
                encoder.write_frame(&frame).unwrap();
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
