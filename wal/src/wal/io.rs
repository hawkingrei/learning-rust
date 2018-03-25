use libc;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
#[derive(Debug)]
struct PosixWritableFile {
    filename_: String,
    use_direct_io_: bool,
    fd_: i32,
    //filesize_: u64,
    //logical_sector_size_: u64,
}

impl PosixWritableFile {
    fn new(filename: String, reopen: bool) -> PosixWritableFile {
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
    fn Allocate(&self,offset :i64,len :i64) {
        unsafe {
            libc::fallocate(self.fd_,libc::FALLOC_FL_KEEP_SIZE,offset,len);
        }
    }
}

#[test]
fn test_append() {
    let mut p = PosixWritableFile::new(String::from("hello"), true);
    p.Append(String::from("hello").into_bytes());
    p.Sync();
    p.Close();
}
