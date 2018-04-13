#[derive(Debug, Clone)]
pub struct EnvOptions {
    // If true, then use mmap to read data
    pub use_mmap_reads: bool,

    // If true, then use mmap to write data
    pub use_mmap_writes: bool,

    // If true, then use O_DIRECT for reading data
    pub use_direct_reads: bool,

    // If true, then use O_DIRECT for writing data
    pub use_direct_writes: bool,

    // If false, fallocate() calls are bypassed
    pub allow_fallocate: bool,

    // If true, set the FD_CLOEXEC on open fd.
    pub set_fd_cloexec: bool,

    // If true, we will preallocate the file with FALLOC_FL_KEEP_SIZE flag, which
    // means that file size won't change as part of preallocation.
    // If false, preallocation will also change the file size. This option will
    // improve the performance in workloads where you sync the data on every
    // write. By default, we set it to true for MANIFEST writes and false for
    // WAL writes
    pub fallocate_with_keep_size: bool,

    pub writable_file_max_buffer_size: usize,

    pub bytes_per_sync: usize,
}

impl Default for EnvOptions {
    fn default() -> EnvOptions {
        EnvOptions {
            use_mmap_reads: false,
            use_mmap_writes: true,
            use_direct_reads: false,
            use_direct_writes: true,
            allow_fallocate: true,
            set_fd_cloexec: true,
            fallocate_with_keep_size: true,

            writable_file_max_buffer_size: 1024 * 1024,
            bytes_per_sync: 0,
        }
    }
}
