use hash::crc32;
use wal;
use wal::env;
use wal::file_reader_writer::SequentialFileReader;
use wal::io::PosixSequentialFile;
use wal::log_format;
use wal::log_format::kBlockSize;
use wal::log_format::kMaxRecordType;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RecordType {
    kEof = kMaxRecordType as isize + 1,
    // Returned whenever we find an invalid physical record.
    // Currently there are three situations in which this happens:
    // * The record has an invalid CRC (ReadPhysicalRecord reports a drop)
    // * The record is a 0-length record (No drop is reported)
    // * The record is below constructor's initial_offset (No drop is reported)
    kBadRecord = kMaxRecordType as isize + 2,
    // Returned when we fail to read a valid header.
    kBadHeader = kMaxRecordType as isize + 3,
    // Returned when we read an old record from a previous user of the log.
    kOldRecord = kMaxRecordType as isize + 4,
    // Returned when we get a bad record length
    kBadRecordLen = kMaxRecordType as isize + 5,
    // Returned when we get a bad record checksum
    kBadRecordChecksum = kMaxRecordType as isize + 6,
}

pub struct Reader {
    eof_: bool,
    buffer_: Vec<u8>,
    backing_store_: Vec<u8>,
    read_error_: bool,

    // Offset of the file position indicator within the last block when an
    // EOF was detected.
    eof_offset_: usize,

    // Offset of the last record returned by ReadRecord.
    last_record_offset_: u64,

    // Offset of the first location past the end of buffer_.
    end_of_buffer_offset_: u64,
    initial_offset_: u64,
    log_number_: u32,
    recycled_: bool,
    checksum_: bool,
    file_: SequentialFileReader<PosixSequentialFile>,
}

impl Reader {
    pub fn new(
        file: SequentialFileReader<PosixSequentialFile>,
        initial_offset: u64,
        log_num: u32,
        checksum: bool,
    ) -> Reader {
        Reader {
            eof_: false,
            buffer_: Vec::new(),
            backing_store_: Vec::with_capacity(log_format::kBlockSize),
            checksum_: checksum,
            eof_offset_: 0,
            last_record_offset_: 0,
            end_of_buffer_offset_: 0,
            initial_offset_: initial_offset,
            read_error_: false,
            file_: file,
            log_number_: log_num,
            recycled_: false,
        }
    }

    fn SkipToInitialBlock(&mut self) -> bool {
        let initial_offset_in_block = self.initial_offset_ % kBlockSize as u64;
        let mut block_start_location = self.initial_offset_ - initial_offset_in_block;

        if (initial_offset_in_block > kBlockSize as u64 - 6) {
            block_start_location += kBlockSize as u64;
        }

        self.end_of_buffer_offset_ = block_start_location;

        if block_start_location > 0 {
            let skip_status = self.file_.Skip(block_start_location as i64);
            if skip_status.isOk() {
                //ReportDrop(static_cast<size_t>(block_start_location), skip_status);
                return false;
            }
        }
        return true;
    }

    // For kAbsoluteConsistency, on clean shutdown we don't expect any error
    // in the log files.  For other modes, we can ignore only incomplete records
    // in the last log file, which are presumably due to a write in progress
    // during restart (or from log recycling).
    //
    // TODO krad: Evaluate if we need to move to a more strict mode where we
    // restrict the inconsistency to only the last log
    pub fn readRecord(
        &mut self,
        mut record: &mut Vec<u8>,
        mut scratch: &mut Vec<u8>,
        wal_recovery_mode: env::WALRecoveryMode,
    ) -> bool {
        if self.last_record_offset_ < self.initial_offset_ {
            if !(self.SkipToInitialBlock()) {
                return false;
            }
        }
        record.clear();
        scratch.clear();
        let mut in_fragmented_record = false;
        // Record offset of the logical record that we're reading
        // 0 is a dummy value to make compilers happy
        let mut prospective_record_offset = 0;

        let mut fragment: Vec<u8> = Vec::new();
        loop {
            let mut physical_record_offset = self.end_of_buffer_offset_ - self.buffer_.len() as u64;
            let mut drop_size: usize = 0;
            let record_type = self.readPhysicalRecord(&mut fragment, &mut drop_size);
            println!("record_type {}", record_type);
            if record_type == log_format::RecordType::kFullType as isize
                || record_type == log_format::RecordType::kRecyclableFullType as isize
            {
                if (in_fragmented_record && !(scratch.len() == 0)) {
                    // Handle bug in earlier versions of log::Writer where
                    // it could emit an empty kFirstType record at the tail end
                    // of a block followed by a kFullType or kFirstType record
                    // at the beginning of the next block.
                    //ReportCorruption(scratch->size(), "partial record without end(1)");
                }
                {
                    prospective_record_offset = physical_record_offset;
                    println!("1 {:?}", fragment);
                    scratch.clear();
                    record.append(&mut fragment);

                    self.last_record_offset_ = prospective_record_offset;
                    return true;
                }
            }

            if record_type == log_format::RecordType::kFirstType as isize
                || record_type == log_format::RecordType::kRecyclableFirstType as isize
            {
                if (in_fragmented_record && !(scratch.len() == 0)) {
                    // Handle bug in earlier versions of log::Writer where
                    // it could emit an empty kFirstType record at the tail end
                    // of a block followed by a kFullType or kFirstType record
                    // at the beginning of the next block.
                    //ReportCorruption(scratch->size(), "partial record without end(1)");
                }
                prospective_record_offset = physical_record_offset;
                println!("2 {:?}", fragment);

                scratch.append(&mut fragment);
                in_fragmented_record = true;
                break;
            }

            if record_type == log_format::RecordType::kMiddleType as isize
                || record_type == log_format::RecordType::kRecyclableMiddleType as isize
            {
                if (in_fragmented_record) {
                    // Handle bug in earlier versions of log::Writer where
                    // it could emit an empty kFirstType record at the tail end
                    // of a block followed by a kFullType or kFirstType record
                    // at the beginning of the next block.
                    //ReportCorruption(scratch->size(), "partial record without end(1)");
                } else {
                    scratch.append(&mut fragment);
                }
                break;
            }

            if record_type == log_format::RecordType::kLastType as isize
                || record_type == log_format::RecordType::kRecyclableLastType as isize
            {
                if (in_fragmented_record) {
                    // Handle bug in earlier versions of log::Writer where
                    // it could emit an empty kFirstType record at the tail end
                    // of a block followed by a kFullType or kFirstType record
                    // at the beginning of the next block.
                    //ReportCorruption(scratch->size(), "partial record without end(1)");
                } else {
                    println!("3 {:?}", fragment);

                    scratch.append(&mut fragment);
                    record.append(&mut fragment);
                    self.last_record_offset_ = prospective_record_offset;
                    return true;
                }
                break;
            }
            println!("{}", 4);
            if record_type == RecordType::kBadHeader as isize {
                if (wal_recovery_mode == env::WALRecoveryMode::kAbsoluteConsistency) {
                    // in clean shutdown we don't expect any error in the log files
                    //ReportCorruption(drop_size, "truncated header");
                }
            }
            println!("{}", 5);
            if record_type == RecordType::kEof as isize {
                if (in_fragmented_record) {
                    if (wal_recovery_mode == env::WALRecoveryMode::kAbsoluteConsistency) {
                        // in clean shutdown we don't expect any error in the log files
                        //ReportCorruption(drop_size, "truncated header");
                    }
                    scratch.clear();
                }
                return false;
            }
            println!("{}", 6);
            if record_type == RecordType::kOldRecord as isize {
                if (wal_recovery_mode != env::WALRecoveryMode::kSkipAnyCorruptedRecords) {
                    // in clean shutdown we don't expect any error in the log files
                    //ReportCorruption(drop_size, "truncated header");
                    if (in_fragmented_record) {
                        if (wal_recovery_mode == env::WALRecoveryMode::kAbsoluteConsistency) {
                            //ReportCorruption(drop_size, "truncated header");
                        }
                        scratch.clear();
                    }
                }
                return false;
            }
            println!("{}", 7);
            if record_type == RecordType::kBadRecord as isize {
                if (in_fragmented_record) {
                    //ReportCorruption(drop_size, "truncated header");
                    in_fragmented_record = false;
                    scratch.clear();
                }
                break;
            }
            println!("{}", 8);
            if record_type == RecordType::kBadRecordLen as isize
                || record_type == RecordType::kBadRecordChecksum as isize
            {
                if (self.recycled_
                    && wal_recovery_mode == env::WALRecoveryMode::kTolerateCorruptedTailRecords)
                {
                    scratch.clear();
                    return false;
                }
                if (record_type == RecordType::kBadRecordLen as isize) {
                    //ReportCorruption(drop_size, "bad record length");
                } else {
                    //ReportCorruption(drop_size, "checksum mismatch");
                }
                if (in_fragmented_record) {
                    //ReportCorruption(scratch->size(), "error in middle of record");
                    in_fragmented_record = false;
                    scratch.clear();
                }
                break;
            } else {
                //char buf[40];
                //snprintf(buf, sizeof(buf), "unknown record type %u", record_type);
                //ReportCorruption((fragment.size() + (in_fragmented_record ? scratch->size() : 0)),buf);
                in_fragmented_record = false;
                scratch.clear();
                break;
            }
        }
        return false;
    }

    fn readPhysicalRecord(&mut self, mut result: &mut Vec<u8>, mut drop_size: &mut usize) -> isize {
        println!("read p ");
        while (true) {
            println!("buf len {}", self.buffer_.len());
            // We need at least the minimum header size
            if (self.buffer_.len() < log_format::kHeaderSize) {
                let mut r: isize = 0;
                if (!self.readMore(&mut drop_size, &mut r)) {
                    return r;
                }
                continue;
            }

            let a = self.buffer_[4] & 0xff;
            let b = self.buffer_[5] & 0xff;
            let log_type = self.buffer_[6];
            println!("type log {}", log_type);
            let length: usize = (a as usize) | ((b as usize) << 8);
            let mut header_size = log_format::kHeaderSize;
            if (log_type >= log_format::RecordType::kRecyclableFullType as u8
                && log_type <= log_format::RecordType::kRecyclableLastType as u8)
            {
                if (self.end_of_buffer_offset_ - self.buffer_.len() as u64 == 0) {
                    self.recycled_ = true;
                }
                header_size = log_format::kRecyclableHeaderSize;
                // We need enough for the larger header
                if (self.buffer_.len() < log_format::kRecyclableHeaderSize) {
                    let mut r = 0;
                    if (!self.readMore(&mut drop_size, &mut r)) {
                        return r;
                    }
                    continue;
                }
                let log_num = wal::DecodeFixed32([
                    self.buffer_[7],
                    self.buffer_[8],
                    self.buffer_[9],
                    self.buffer_[10],
                ]);
                if (log_num != self.log_number_) {
                    return RecordType::kOldRecord as isize;
                }
            }
            if (header_size + length > self.buffer_.len()) {
                *drop_size = self.buffer_.len();
                self.buffer_.clear();
                if (!self.eof_) {
                    return RecordType::kBadRecordLen as isize;
                }
                // If the end of the file has been reached without reading |length| bytes
                // of payload, assume the writer died in the middle of writing the record.
                // Don't report a corruption unless requested.
                if (*drop_size > 0) {
                    return RecordType::kBadHeader as isize;
                }
                return RecordType::kEof as isize;
            }

            if (log_type == log_format::RecordType::kZeroType as u8 && length == 0) {
                // Skip zero length record without reporting any drops since
                // such records are produced by the mmap based writing code in
                // env_posix.cc that preallocates file regions.
                // NOTE: this should never happen in DB written by new RocksDB versions,
                // since we turn off mmap writes to manifest and log files
                self.buffer_.clear();
                return RecordType::kBadRecord as isize;
            }

            if (self.checksum_) {
                let expected_crc = wal::DecodeFixed32([
                    self.buffer_[0],
                    self.buffer_[1],
                    self.buffer_[2],
                    self.buffer_[3],
                ]);
                let actual_crc = crc32(0, &self.buffer_[6..length + header_size]); //?
                if (actual_crc != expected_crc) {
                    // Drop the rest of the buffer since "length" itself may have
                    // been corrupted and if we trust it, we could find some
                    // fragment of a real log record that just happens to look
                    // like a valid log record.
                    *drop_size = self.buffer_.len();
                    self.buffer_.clear();
                    return RecordType::kBadRecordChecksum as isize;
                }
            }
            self.buffer_ = self.buffer_[header_size + length..].to_vec();
            if ((self.end_of_buffer_offset_ - self.buffer_.len() as u64 - header_size as u64
                - length as u64) < self.initial_offset_)
            {
                result.clear();
                return RecordType::kBadHeader as isize;
            }

            let result = &mut self.buffer_
                .get(header_size - 1..header_size - 1 + length)
                .unwrap()
                .to_vec();

            return log_type as isize;
        }
        return 0;
    }

    fn readMore(&mut self, mut drop_size: &mut usize, mut error: &mut isize) -> bool {
        if (!self.eof_ && !self.read_error_) {
            self.buffer_.clear();
            let s = self.file_.Read(
                log_format::kBlockSize,
                &mut self.buffer_,
                &mut self.backing_store_,
            );
            println!("readMore buf len {}", self.buffer_.len());
            self.end_of_buffer_offset_ += self.buffer_.len() as u64;
            if (!s.isOk()) {
                println!("read fail");
                self.buffer_.clear();
                //ReportDrop(kBlockSize, status);
                self.read_error_ = true;
                *error = RecordType::kEof as isize;
                return false;
            } else {
                println!("read ok");
                if self.buffer_.len() < log_format::kBlockSize {
                    self.eof_ = true;
                    self.eof_offset_ = self.buffer_.len();
                }
            }
            return true;
        } else {
            if (self.buffer_.len() > 0) {
                *drop_size = self.buffer_.len();
                self.buffer_.clear();
                *error = RecordType::kBadHeader as isize;
                return false;
            }
            self.buffer_.clear();
            *error = RecordType::kEof as isize;
            return false;
        }
    }

    //fn ReportDrop(bytes: usize, reason: state) {}
}
