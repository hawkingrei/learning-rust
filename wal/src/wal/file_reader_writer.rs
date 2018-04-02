use std::cmp::min;
use std::mem;
use wal::WritableFile;
use wal::aligned_buffer::AlignedBuffer;
use wal::env::EnvOptions;
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
    pub fn new(writable_file: T, options: EnvOptions) -> WritableFileWriter<T> {
        WritableFileWriter {
            writable_file_: writable_file,
            filesize_: 0,
            max_buffer_size_: options.writable_file_max_buffer_size,
            pending_sync_: false,
            bytes_per_sync_: options.bytes_per_sync,
            buf_: Default::default(),
            last_sync_size_: 0,
        }
    }

    pub fn append(&mut self, slice: Vec<u8>) -> state {
        let mut s: state = state::ok();
        let mut src = 0;
        let mut ptr = slice.as_slice();
        let mut left = mem::size_of_val(&slice.as_slice());
        self.pending_sync_ = true;
        {
            let fsize = self.get_file_size();
            self.writable_file_.prepare_write(fsize, left);
        }
        if (self.buf_.get_capacity() - self.buf_.get_current_size() < left) {
            println!("1");
            let mut cap = self.buf_.get_capacity();
            println!("cap {} max_buffer_size_ {}",cap,self.max_buffer_size_);
            while (cap < self.max_buffer_size_) {
                println!("cap {} max_buffer_size_ {}",cap,self.max_buffer_size_);
                // See whether the next available size is large enough.
                // Buffer will never be increased to more than max_buffer_size_.
                let desired_capacity = min(cap * 2, self.max_buffer_size_);
                if desired_capacity - self.buf_.get_current_size() >= left
                    || (self.writable_file_.use_direct_io()
                        && desired_capacity == self.max_buffer_size_)
                {
                    self.buf_.allocate_new_buffer(desired_capacity, true);
                    break;
                }
                cap *= 2;
            }
        }
        // Flush only when buffered I/O
        if !self.writable_file_.use_direct_io()
            && (self.buf_.get_capacity() - self.buf_.get_current_size() < left)
        {
            let s: state;
            if (self.buf_.get_current_size() > 0) {
                s = self.flush();
                if !s.isOk() {
                    return s;
                }
            }
            assert!(self.buf_.get_current_size() == 0);
        }

        // We never write directly to disk with direct I/O on.
        // or we simply use it for its original purpose to accumulate many small
        // chunks
        println!("cap {} left {}",self.buf_.get_capacity(),left);
        if (self.writable_file_.use_direct_io() || self.buf_.get_capacity() > left) {
            println!("3");
            while (left > 0) {
                let appended = self.buf_.append(slice[src..].to_vec(), left);
                left -= appended;
                src += appended;
                if (left > 0) {
                    s = self.flush();
                    if (!s.isOk()) {
                        break;
                    }
                }
            }
        } else {
            assert!(self.buf_.get_current_size() == 0);
            s = self.write_buffered(slice[src..].to_vec(), left);
        }

        if (s.isOk()) {
            self.filesize_ = mem::size_of_val(&slice.as_slice());
        }
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
        return self.writable_file_.range_sync(offset, nbytes);
    }

    fn write_buffered(&mut self, data: Vec<u8>, size: usize) -> state {
        let mut s: state;
        assert!(self.writable_file_.use_direct_io());
        let mut src = 0;
        let mut left = size;
        println!("write buffered {}",left);
        while (left > 0) {
            let mut allowed;

            // if (rate_limiter_ != nullptr) {
            // allowed = rate_limiter_->RequestToken(
            // left, 0 /* alignment */, writable_file_->GetIOPriority(), stats_,
            // RateLimiter::OpType::kWrite);
            // } else {
            allowed = left;
            // }
            s = self.writable_file_.append(data[src..src + left].to_vec());
            if (!s.isOk()) {
                return s;
            }

            left -= allowed;
            src += allowed;
        }
        self.buf_.size(0);
        state::ok()
    }
}
