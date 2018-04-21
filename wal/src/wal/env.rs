#[derive(PartialEq)]
pub enum WALRecoveryMode {
    // Original levelDB recovery
    // We tolerate incomplete record in trailing data on all logs
    // Use case : This is legacy behavior
    kTolerateCorruptedTailRecords = 0x00,
    // Recover from clean shutdown
    // We don't expect to find any corruption in the WAL
    // Use case : This is ideal for unit tests and rare applications that
    // can require high consistency guarantee
    kAbsoluteConsistency = 0x01,
    // Recover to point-in-time consistency (default)
    // We stop the WAL playback on discovering WAL inconsistency
    // Use case : Ideal for systems that have disk controller cache like
    // hard disk, SSD without super capacitor that store related data
    kPointInTimeRecovery = 0x02,
    // Recovery after a disaster
    // We ignore any corruption in the WAL and try to salvage as much data as
    // possible
    // Use case : Ideal for last ditch effort to recover data or systems that
    // operate with low grade unrelated data
    kSkipAnyCorruptedRecords = 0x03,
}

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
