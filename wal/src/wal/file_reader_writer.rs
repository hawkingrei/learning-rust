use std::mem;
use wal::WritableFile;
use wal::io;
use wal::state;

#[derive(Debug)]
pub struct WritableFileWriter<T: WritableFile> {
    writable_file_: T,
    filesize_: usize,
    max_buffer_size_: isize,
    pending_sync_: bool,
    // AlignedBuffer           buf_;
    // uint64_t                last_sync_size_;
    // uint64_t                bytes_per_sync_;
    // RateLimiter*            rate_limiter_;
    // Statistics* stats_;
}

impl<T: WritableFile> WritableFileWriter<T> {
    pub fn append(&mut self, slice: Vec<u8>) -> state {
        let mut ptr = slice.as_slice();
        let mut left = mem::size_of_val(&slice.as_slice());
        self.pending_sync_ = true;
        {
            let fsize = self.get_file_size();
            self.writable_file_.prepare_write(fsize, left);
        }
        if (self.writable_file_.use_direct_io()) {}
        state::ok()
    }

    fn get_file_size(&self) -> usize {
        return self.filesize_;
    }

    pub fn flush(&self) -> state {
        return state::ok();
    }
}
