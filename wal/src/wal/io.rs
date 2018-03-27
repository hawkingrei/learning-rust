use libc;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
use wal::WritableFile;
#[derive(Debug)]
struct PosixWritableFile {
    filename_: String,
    use_direct_io_: bool,
    fd_: i32,

    preallocation_block_size_: usize,
    last_preallocated_block_: usize,
    //filesize_: u64,
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
        }
    }

    fn Append(&self, data: Vec<u8>) {
        unsafe {
            libc::write(
                self.fd_,
                data.as_ptr() as *const libc::c_void,
                mem::size_of_val(data.as_slice()),
            );
        }
    }

    fn Sync(&self) {
        unsafe {
            libc::fsync(self.fd_);
        }
    }

    fn Close(&self) {
        unsafe {
            libc::close(self.fd_);
        }
    }

    #[cfg(target_os = "linux")]
    fn Allocate(&self, offset: i64, len: i64) {
        unsafe {
            libc::fallocate(self.fd_, libc::FALLOC_FL_KEEP_SIZE, offset, len);
        }
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
            self.Allocate(
                (block_size * self.last_preallocated_block_) as i64,
                (block_size * num_spanned_blocks) as i64,
            );
            self.last_preallocated_block_ = new_last_preallocated_block;
        }
    }
}

#[test]
fn test_append() {
    let p = PosixWritableFile::new(String::from("hello"), true, 20);
    p.Append(String::from("hello").into_bytes());
    p.Sync();
    p.Close();
}
