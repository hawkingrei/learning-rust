use wal::log_format::{RecordType, kBlockSize, kHeaderSize, kRecyclableHeaderSize};
use std::mem;

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
    fn add_record(&mut self, slice: Vec<u8>) {
        /*
        const char* ptr = slice.data();
        size_t left = slice.size();
        */
        let mut ptr = slice.as_slice();
        let left = mem::size_of_val(&slice.as_slice());
        let header_size = if self.recycle_log_files_ {
            kRecyclableHeaderSize
        } else {
            kHeaderSize
        };

        let mut begin = true;
        let mut fragment_length: usize;
        loop {
            let leftover: usize = kBlockSize - self.block_offset_;
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
            assert!((kBlockSize - self.block_offset_) >= header_size);
            let avail: usize = kBlockSize - self.block_offset_ - header_size;
            fragment_length = if left < avail { left } else { avail };

            let end: bool = (left == fragment_length);
            if (begin && end) {
                let mut rtype: RecordType = if self.recycle_log_files_ {
                    RecordType::kRecyclableFullType
                } else {
                    RecordType::kFullType
                };
            } else if (begin) {
                let mut rtype: RecordType = if self.recycle_log_files_ {
                    RecordType::kRecyclableFirstType
                } else {
                    RecordType::kFirstType
                };
            } else if (end) {
                let mut rtype: RecordType = if self.recycle_log_files_ {
                    RecordType::kRecyclableLastType
                } else {
                    RecordType::kLastType
                };
            } else {
                let mut rtype: RecordType = if self.recycle_log_files_ {
                    RecordType::kRecyclableMiddleType
                } else {
                    RecordType::kMiddleType
                };
            };
        }
        ptr = &ptr[fragment_length..];
        left -= fragment_length;
    }

    fn EmitPhysicalRecord(t: RecordType, ptr: Vec<u8>, n: usize) {
        let header_size: usize = 0;
        let mut buf: [u8; kRecyclableHeaderSize] = [0u8; kRecyclableHeaderSize];
    }
}
