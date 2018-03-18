use wal::log_format::{kRecyclableHeaderSize,kHeaderSize,kBlockSize};

#[derive(Debug)]
struct Write {
    block_offset_: usize, // Current offset in block
    log_number_: u64,
    recycle_log_files_: bool,
    manual_flush_: bool,
}

impl Write {
    fn new(log_number: u64, recycle_log_files: bool, manual_flush: bool) -> Write {
        Write {
            block_offset_: 0,
            log_number_: log_number,
            recycle_log_files_: recycle_log_files,
            manual_flush_: manual_flush,
        }
    }
    /*const Slice& slice*/
    fn add_record(&mut self) {
        /*
        const char* ptr = slice.data();
        size_t left = slice.size();
        */
        let header_size = if self.recycle_log_files_ {
            kRecyclableHeaderSize
        } else {
            kHeaderSize
        };

        let mut begin = true;
        loop {
            let leftover :usize = kBlockSize - self.block_offset_;
            assert!(leftover >= 0);
            
            if (leftover < header_size) {
                if (leftover > 0) {
                assert!(header_size <= 11);
                /*
                dest_->Append(
                    Slice("\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
                    static_cast<size_t>(leftover)));
                */
                }
                self.block_offset_ = 0;
            }
            
            let avail = kBlockSize - self.block_offset_ - header_size;
            
            
        }
    }
}
