use libc;
use libc::c_int;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
use std::ptr;
use std::slice;
use std::usize;
use wal::env;
use wal::io;
use wal::k_default_page_size;
use wal::state;
use wal::Code;
use wal::SequentialFile;
use wal::WritableFile;

pub fn clearerr(stream: *mut libc::FILE) {
    extern "C" {
        fn clearerr(stream: *mut libc::FILE);
    }
    unsafe {
        clearerr(stream);
    }
}

#[cfg(any(target_os = "macos"))]
unsafe fn fread_unlocked(
    ptr: *mut libc::c_void,
    size: libc::size_t,
    nobj: libc::size_t,
    stream: *mut libc::FILE,
) -> libc::size_t {
    return libc::fread(ptr, size, nobj, stream);
}

#[cfg(any(target_os = "linux"))]
extern "C" {
    fn posix_fread_unlocked(
        __ptr: *mut libc::c_void,
        __size: libc::size_t,
        __n: libc::size_t,
        __stream: *mut libc::FILE,
    ) -> libc::size_t;
}

#[cfg(any(target_os = "linux"))]
unsafe fn fread_unlocked(
    ptr: *mut libc::c_void,
    size: libc::size_t,
    nobj: libc::size_t,
    stream: *mut libc::FILE,
) -> libc::size_t {
    return posix_fread_unlocked(ptr, size, nobj, stream);
}

fn SetFD_CLOEXEC(fd: i32, options: env::EnvOptions) {
    if (options.set_fd_cloexec && fd > 0) {
        unsafe {
            libc::fcntl(
                fd,
                libc::F_SETFD,
                libc::fcntl(fd, libc::F_GETFD) | libc::FD_CLOEXEC,
            );
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
unsafe fn errno_location() -> *const c_int {
    extern "C" {
        fn __error() -> *const c_int;
    }
    __error()
}

#[cfg(target_os = "bitrig")]
fn errno_location() -> *const c_int {
    extern "C" {
        fn __errno() -> *const c_int;
    }
    unsafe { __errno() }
}

#[cfg(target_os = "dragonfly")]
unsafe fn errno_location() -> *const c_int {
    extern "C" {
        fn __dfly_error() -> *const c_int;
    }
    __dfly_error()
}

#[cfg(target_os = "openbsd")]
unsafe fn errno_location() -> *const c_int {
    extern "C" {
        fn __errno() -> *const c_int;
    }
    __errno()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn errno_location() -> *const c_int {
    extern "C" {
        fn __errno_location() -> *const c_int;
    }
    __errno_location()
}

#[derive(Debug)]
pub struct PosixWritableFile {
    filename_: String,
    use_direct_io_: bool,
    fd_: i32,
    preallocation_block_size_: usize,
    last_preallocated_block_: usize,
    filesize_: usize,
    logical_sector_size_: usize,
}

#[cfg(target_os = "macos")]
fn get_flag() -> i32 {
    libc::O_CREAT
}

#[cfg(
    any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd"
    )
)]
fn get_flag() -> i32 {
    libc::O_CREAT | libc::O_DIRECT
}

fn get_logical_buffer_size() -> usize {
    if cfg!(not(target_os = "linux")) {
        return k_default_page_size;
    } else {
        return k_default_page_size;
        //Todo: support linux
    }
}

fn IsSectorAligned(off: usize, sector_size: usize) -> bool {
    return off % sector_size == 0;
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
            logical_sector_size_: get_logical_buffer_size(),
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

    fn get_required_buffer_alignment(&self) -> usize {
        self.logical_sector_size_
    }

    fn positioned_append(&mut self, mut data: Vec<u8>, mut offset: usize) -> state {
        if (self.use_direct_io()) {
            //println!("offset {} get_logical_buffer_size {}",offset,get_logical_buffer_size());
            //assert!(IsSectorAligned(offset, get_logical_buffer_size()));
            //println!("data len {} get_logical_buffer_size {}",data.len(),get_logical_buffer_size());
            //assert!(IsSectorAligned(data.len(), get_logical_buffer_size()));
            //assert!(IsSectorAligned(data.as_ptr() as usize,get_logical_buffer_size()));
        }
        assert!(offset <= usize::MAX);
        let mut src = data.as_mut_ptr();
        let mut left = data.len();

        let mut done;
        while (left != 0) {
            unsafe {
                done = libc::pwrite(self.fd_, src as *const libc::c_void, left, offset as i64);
            }
            if done < 1 {
                unsafe {
                    if (*errno_location()) as i32 == libc::EINTR {
                        continue;
                    }
                }
                return state::new(
                    Code::kIOError,
                    format!("While pwrite to file at offset {}", offset.to_string()),
                    "".to_string(),
                );
                //IOError("While pwrite to file at offset " + ToString(offset),filename_, errno);
            }
            left -= done as usize;
            offset += done as usize;
            unsafe {
                src = src.offset(done);
            }
        }
        self.filesize_ = offset;
        return state::ok();
    }
}

#[test]
fn test_append() {
    let mut p = PosixWritableFile::new(String::from("hello"), true, 20);
    p.append(String::from("hello").into_bytes());
    p.sync();
}

#[cfg(target_os = "macos")]
fn get_flag_for_posix_sequential_file() -> i32 {
    0
}

#[cfg(
    any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd"
    )
)]
fn get_flag_for_posix_sequential_file() -> i32 {
    libc::O_DIRECT
}

#[derive(Debug)]
pub struct PosixSequentialFile {
    filename_: String,
    fd_: i32,
    use_direct_io_: bool,
    logical_sector_size_: usize,
    file_: *mut libc::FILE,
}

impl SequentialFile for PosixSequentialFile {
    fn new(filename: String, options: env::EnvOptions, ptr: &mut PosixSequentialFile) -> state {
        let mut fd = -1;
        let mut flag = libc::O_RDONLY;
        let mut file = 0 as *mut libc::FILE;
        if (options.use_direct_reads && !options.use_mmap_reads) {
            if cfg!(feature = "CIBO_LITE") {
                return state::new(
                    Code::kIOError,
                    "Direct I/O not supported in cibo lite".to_string(),
                    "".to_string(),
                );
            }
            flag = flag | get_flag_for_posix_sequential_file();
        }
        flag = get_flag_for_posix_sequential_file();
        loop {
            unsafe {
                fd = libc::open(
                    CString::from_vec_unchecked(filename.clone().into_bytes()).as_ptr(),
                    flag,
                    0o644,
                );
                if !(fd < 0) && *errno_location() as i32 == libc::EINTR {
                    break;
                }
            }
        }
        if fd < 0 {
            return state::new(
                Code::kIOError,
                "While opening a file for sequentially reading".to_string(),
                "".to_string(),
            );
        }

        SetFD_CLOEXEC(fd, options.clone());
        if (options.use_direct_reads && !options.use_mmap_reads) {
            #[cfg(target_os = "macos")]
            unsafe {
                if (libc::fcntl(fd, libc::F_NOCACHE, 1) == -1) {
                    libc::close(fd);
                    //return IOError("While fcntl NoCache", fname, errno);
                }
            }
        } else {
            unsafe {
                loop {
                    file = libc::fdopen(fd, &('r' as libc::c_char));
                    if !(file == 0 as *mut libc::FILE && *errno_location() as i32 == libc::EINTR) {
                        break;
                    }
                }
                if file == 0 as *mut libc::FILE {
                    libc::close(fd);
                    return state::new(
                        Code::kIOError,
                        "While opening a file for sequentially read".to_string(),
                        "".to_string(),
                    );
                }
            }
        }
        *ptr = PosixSequentialFile {
            filename_: filename,
            fd_: fd,
            file_: file,
            use_direct_io_: true,
            logical_sector_size_: get_logical_buffer_size(),
        };
        return state::ok();
    }

    fn Skip(&self, n: i64) -> state {
        unsafe {
            if (libc::fseek(self.file_, n, libc::SEEK_CUR) > 0) {
                // return IOError("While fseek to skip " + ToString(n) + " bytes", filename_, errno);
                return state::new(
                    Code::kIOError,
                    "While fseek to skip ".to_string() + &n.to_string() + &" bytes".to_string(),
                    "".to_string(),
                );
            }
            return state::ok();
        }
    }

    fn Read(&mut self, n: usize, result: &mut Vec<u8>, scratch: *mut libc::c_void) -> state {
        let mut s: state = state::ok();
        let r: usize = 0;
        unsafe {
            loop {
                let r = fread_unlocked(scratch, 1, n, self.file_);
                if !(libc::ferror(self.file_) > 0 && ((*errno_location()) as i32 == libc::EINTR)
                    && r == 0)
                {
                    break;
                }
            }

            *result = Vec::from_raw_parts(scratch as *mut u8, r as usize, r as usize).to_vec();
            if (r < n) {
                if libc::feof(self.file_) > 0 {
                    clearerr(self.file_);
                } else {
                    s = state::new(
                        Code::kIOError,
                        "While reading file sequentially".to_string(),
                        "".to_string(),
                    );
                }
            }
        }
        return s;
    }
}
