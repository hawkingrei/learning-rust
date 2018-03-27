use wal::WritableFile;

#[derive(Debug)]
struct WritableFileWriter {
    writable_file_ : WritableFile;
    max_buffer_size_ :isize; 
    pending_sync_ :bool;   
}   

impl WritableFileWriter {
    fn append(&mut self, slice: Vec<u8>) {
        let mut ptr = slice.as_slice();
        let mut left = mem::size_of_val(&slice.as_slice());
        self.pending_sync_ = true
        
    }
}