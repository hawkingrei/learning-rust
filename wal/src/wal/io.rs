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

impl WritableFile for PosixWritableFile {
    fn new(filename: String, reopen: bool, preallocation_block_size: usize) -> PosixWritableFile {
        let fd;
        unsafe {
            fd = libc::open(
                CString::from_vec_unchecked(filename.clone().into_bytes()).as_ptr(),
                if reopen {
                    libc::O_CREAT | libc::O_APPEND | libc::O_RDWR
                } else {
                    libc::O_CREAT | libc::O_TRUNC | libc::O_RDWR
                },
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

    fn append(&mut self, data: Vec<u8>) -> Result<state, state> {
        let state: isize;
        unsafe {
            state = libc::write(
                self.fd_,
                data.as_ptr() as *const libc::c_void,
                mem::size_of_val(data.as_slice()),
            );
        }
        if state < 0 {
            return Err(state::new(
                Code::kIOError,
                "cannot append".to_string(),
                "".to_string(),
            ));
        }
        self.filesize_ += mem::size_of_val(data.as_slice());
        return Ok(state::ok());
    }

    fn sync(&self) -> Result<state, state> {
        let state: i32;
        unsafe {
            state = libc::fsync(self.fd_);
        }
        if state < 0 {
            return Err(state::new(
                Code::kIOError,
                "cannot sync".to_string(),
                "".to_string(),
            ));
        }
        return Ok(state::ok());
    }

    fn close(&self) -> Result<state, state> {
        let state: i32;
        unsafe {
            state = libc::close(self.fd_);
        }
        if state < 0 {
            return Err(state::new(
                Code::kIOError,
                "cannot close".to_string(),
                "".to_string(),
            ));
        }
        return Ok(state::ok());
    }

    #[cfg(target_os = "linux")]
    fn sync_file_range(&self, offset: i64, nbytes: i64) -> Result<state, state> {
        let state: i32;
        unsafe {
            state = libc::sync_file_range(self.fd_, offset, nbytes, libc::SYNC_FILE_RANGE_WRITE);
        }
        if state < 0 {
            return Err(state::new(
                Code::kIOError,
                "cannot sync_file_range".to_string(),
                "".to_string(),
            ));
        }
        return Ok(state::ok());
    }

    #[cfg(target_os = "linux")]
    fn allocate(&self, offset: i64, len: i64) -> Result<state, state> {
        let state: i32;
        unsafe {
            state = libc::fallocate(self.fd_, libc::FALLOC_FL_KEEP_SIZE, offset, len);
        }
        if state < 0 {
            return Err(state::new(
                Code::kIOError,
                "cannot allocate".to_string(),
                "".to_string(),
            ));
        }
        return Ok(state::ok());
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

    fn use_direct_io(&self) -> bool {
        return self.use_direct_io_;
    }
}

#[test]
fn test_append() {
    let mut p = PosixWritableFile::new(String::from("hello"), true, 20);
    p.append(String::from("hello").into_bytes());
    p.sync();
    p.close();
}
