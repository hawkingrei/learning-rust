use std::fs::File;
use std::io::prelude::*;

fn main() {
	let mut f = File::create("foo.txt");
	let mut f = match f {
        	Ok(file) => file,
        	Err(e) => {
         	   	panic!("Failed to open a file: {:?}", e);
        	}
    	};
	f.write_all(b"Hello, world!");
	f.sync_all();
}
