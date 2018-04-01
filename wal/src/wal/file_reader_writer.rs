use std::cmp::min;
use std::mem;
use wal::WritableFile;
use wal::aligned_buffer::AlignedBuffer;
use wal::io;
use wal::state;

#[derive(Debug)]
pub struct WritableFileWriter<T: WritableFile> {
    writable_file_: T,
    filesize_: usize,
    max_buffer_size_: usize,
    pending_sync_: bool,
    buf_: AlignedBuffer,
    bytes_per_sync_: usize,
    last_sync_size_: usize,
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

        if (self.buf_.get_capacity() - self.buf_.get_current_size() < left) {
            let mut cap = self.buf_.get_capacity();
            while (cap < self.max_buffer_size_) {
                let desired_capacity = min(cap * 2, self.max_buffer_size_);
                if (desired_capacity - self.buf_.get_current_size() >= left
                    || (self.writable_file_.use_direct_io()
                        && desired_capacity == self.max_buffer_size_))
                {
                    self.buf_.allocate_new_buffer(desired_capacity, true);
                    break;
                }
                cap *= 2;
            }
        }

        if (!self.writable_file_.use_direct_io()
            && (self.buf_.get_capacity() - self.buf_.get_current_size() < left))
        {}

        state::ok()
    }

    fn get_file_size(&self) -> usize {
        return self.filesize_;
    }

    pub fn flush(&mut self) -> state {
        let mut s: state;
        if (self.buf_.get_current_size() > 0) {
            if cfg!(feature = "CIBO_LITE") {}
        } else {

        }
        s = self.writable_file_.flush();
        if (!s.isOk()) {
            return s;
        }

        // sync OS cache to disk for every bytes_per_sync_
        // TODO: give log file and sst file different options (log
        // files could be potentially cached in OS for their whole
        // life time, thus we might not want to flush at all).

        // We try to avoid sync to the last 1MB of data. For two reasons:
        // (1) avoid rewrite the same page that is modified later.
        // (2) for older version of OS, write can block while writing out
        //     the page.
        // Xfs does neighbor page flushing outside of the specified ranges. We
        // need to make sure sync range is far from the write offset.
        if (!self.writable_file_.use_direct_io() && self.bytes_per_sync_ > 0) {
            let k_bytes_not_sync_range: usize = 1024 * 1024;
            let k_bytes_align_when_sync: usize = 4 * 1024;
            if (self.filesize_ > k_bytes_not_sync_range) {
                let mut offset_sync_to = self.filesize_ - k_bytes_not_sync_range;
                offset_sync_to -= offset_sync_to % k_bytes_align_when_sync;
                assert!(offset_sync_to >= self.last_sync_size_);
                if (offset_sync_to > 0
                    && offset_sync_to - self.last_sync_size_ >= self.bytes_per_sync_)
                {
                    let last_sync_size_ = self.last_sync_size_;
                    s = self.range_sync(
                        last_sync_size_ as i64,
                        (offset_sync_to - last_sync_size_) as i64,
                    );
                    self.last_sync_size_ = offset_sync_to;
                }
            }
        }
        s
    }

    fn range_sync(&mut self, offset: i64, nbytes: i64) -> state {
        return self.range_sync(offset, nbytes);
    }
}
