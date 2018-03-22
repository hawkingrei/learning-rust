use wal::log_format::{RecordType, kBlockSize, kHeaderSize, kMaxRecordType, kRecyclableHeaderSize};
use std::mem;
use hash::crc32;
use wal;

#[derive(Debug)]
struct Write {
    block_offset_: usize, // Current offset in block
    log_number_: u64,
    recycle_log_files_: bool,
    manual_flush_: bool,
    type_crc_: Vec<u32>,
}

impl Write {
    fn new(log_number: u64, recycle_log_files: bool, manual_flush: bool) -> Write {
        let mut type_crc: [u32; kMaxRecordType as usize + 1] = [0u32; kMaxRecordType as usize + 1];
        for x in 0..kMaxRecordType + 1 {
            type_crc[x as usize] = crc32(0, &[x]);
        }
        Write {
            block_offset_: 0,
            log_number_: log_number,
            recycle_log_files_: recycle_log_files,
            manual_flush_: manual_flush,
            type_crc_: type_crc.to_vec(),
        }
    }
    /*const Slice& slice*/
    fn add_record(&mut self, slice: Vec<u8>) {
        /*
        const char* ptr = slice.data();
        size_t left = slice.size();
        */
        let mut ptr = slice.as_slice();
        let mut left = mem::size_of_val(&slice.as_slice());
        let header_size = if self.recycle_log_files_ {
            kRecyclableHeaderSize
        } else {
            kHeaderSize
        };

        loop {
            let mut begin = true;
            let mut fragment_length: usize;
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
            ptr = &ptr[fragment_length..];
            left -= fragment_length;

            if left <= 0 {
                break;
            }
        }
    }

    fn EmitPhysicalRecord(&self, t: RecordType, ptr: Vec<u8>, n: usize) {
        let mut header_size: usize = 0;
        let mut buf: [u8; kRecyclableHeaderSize] = [0u8; kRecyclableHeaderSize];
        let mut crc = self.type_crc_[t as usize];

        buf[4] = (n & 0xffusize) as u8;
        buf[5] = (n >> 8) as u8;
        buf[6] = t as u8;

        if ((t as u8) < RecordType::kRecyclableFullType as u8) {
            header_size = kHeaderSize;
            crc = crc32(crc, &buf[4..kHeaderSize]);
        } else {
            header_size = kRecyclableHeaderSize;
            let lnSlice = wal::EncodeFixed64(self.log_number_);
            buf[7] = lnSlice[0];
            buf[8] = lnSlice[1];
            buf[9] = lnSlice[2];
            buf[10] = lnSlice[3];
            crc = crc32(crc, &buf[4..kRecyclableHeaderSize]);
        }
        crc = crc32(crc, &ptr.as_slice());
        buf[..4].clone_from_slice(&wal::EncodeFixed32(crc));
    }
}
