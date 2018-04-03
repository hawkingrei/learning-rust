use libc;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
use wal::Code;
use wal::WritableFile;
use wal::state;
#[derive(Debug)]
pub struct PosixWritableFile {
    filename_: String,
    use_direct_io_: bool,
    fd_: i32,
    preallocation_block_size_: usize,
    last_preallocated_block_: usize,
    filesize_: usize,
    //logical_sector_size_: u64,
}

#[cfg(target_os = "macos")]
fn get_flag() -> i32 {
    libc::O_CREAT
}

#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd",
          target_os = "linux", target_os = "netbsd"))]
fn get_flag() -> i32 {
    libc::O_CREAT | libc::O_DIRECT
}

impl WritableFile for PosixWritableFile {
    fn new(filename: String, reopen: bool, preallocation_block_size: usize) -> PosixWritableFile {
        let fd;
        let flag = if reopen {
            get_flag() | libc::O_APPEND | libc::O_RDWR
        } else {
            get_flag() | libc::O_TRUNC | libc::O_RDWR
        };
        unsafe {
            fd = libc::open(
                CString::from_vec_unchecked(filename.clone().into_bytes()).as_ptr(),
                flag,
                0o644,
            );
        }
        PosixWritableFile {
            filename_: filename,
            use_direct_io_: true,
            fd_: fd,
            preallocation_block_size_: preallocation_block_size,
            last_preallocated_block_: 0,
            filesize_: 0,
        }
    }

    fn append(&mut self, data: Vec<u8>) -> state {
        let state: isize;
        unsafe {
            state = libc::write(
                self.fd_,
                data.as_ptr() as *const libc::c_void,
                mem::size_of_val(data.as_slice()),
            );
        }
        if state < 0 {
            return state::new(Code::kIOError, "cannot append".to_string(), "".to_string());
        }
        self.filesize_ += mem::size_of_val(data.as_slice());
        return state::ok();
    }

    fn sync(&self) -> state {
        let state: i32;
        unsafe {
            state = libc::fsync(self.fd_);
        }
        if state < 0 {
            return state::new(Code::kIOError, "cannot sync".to_string(), "".to_string());
        }
        return state::ok();
    }

    fn close(&self) -> state {
        let state: i32;
        unsafe {
            state = libc::close(self.fd_);
        }
        if state < 0 {
            return state::new(Code::kIOError, "cannot close".to_string(), "".to_string());
        }
        return state::ok();
    }

    #[cfg(target_os = "linux")]
    fn range_sync(&self, offset: i64, nbytes: i64) -> state {
        let state: i32;
        unsafe {
            state = libc::sync_file_range(self.fd_, offset, nbytes, libc::SYNC_FILE_RANGE_WRITE);
        }
        if state < 0 {
            return state::new(
                Code::kIOError,
                "cannot sync_file_range".to_string(),
                "".to_string(),
            );
        }
        return state::ok();
    }

    #[cfg(target_os = "linux")]
    fn allocate(&self, offset: i64, len: i64) -> state {
        let state: i32;
        unsafe {
            state = libc::fallocate(self.fd_, libc::FALLOC_FL_KEEP_SIZE, offset, len);
        }
        if state < 0 {
            return state::new(
                Code::kIOError,
                "cannot allocate".to_string(),
                "".to_string(),
            );
        }
        return state::ok();
    }

    #[cfg(target_os = "linux")]
    fn prepare_write(&mut self, offset: usize, len: usize) {
        if (self.preallocation_block_size_ == 0) {
            return;
        }
        let block_size = self.preallocation_block_size_;
        let new_last_preallocated_block = (offset + len + block_size - 1) / block_size;
        if (new_last_preallocated_block > self.last_preallocated_block_) {
            let num_spanned_blocks = new_last_preallocated_block - self.last_preallocated_block_;
            self.allocate(
                (block_size * self.last_preallocated_block_) as i64,
                (block_size * num_spanned_blocks) as i64,
            );
            self.last_preallocated_block_ = new_last_preallocated_block;
        }
    }

    fn flush(&self) -> state {
        return state::ok();
    }

    fn use_direct_io(&self) -> bool {
        return self.use_direct_io_;
    }

    fn fcntl(&self) -> bool {
        return unsafe { libc::fcntl(self.fd_, libc::F_GETFL) != -1 };
    }

    fn truncate(&mut self, size: usize) -> state {
        let state: i32;
        unsafe {
            state = libc::ftruncate(self.fd_, size as i64);
        }
        if state < 0 {
            return state::new(
                Code::kIOError,
                "cannot truncate".to_string(),
                "".to_string(),
            );
        } else {
            self.filesize_ = size;
        }
        return state::ok();
    }
}

#[test]
fn test_append() {
    let mut p = PosixWritableFile::new(String::from("hello"), true, 20);
    p.append(String::from("hello").into_bytes());
    p.sync();
}
