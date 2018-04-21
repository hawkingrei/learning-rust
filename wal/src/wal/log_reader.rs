use wal;
use wal::file_reader_writer::SequentialFileReader;
use wal::io::PosixSequentialFile;
use wal::log_format::kBlockSize;
use wal::log_format::kMaxRecordType;
#[derive(Clone, Copy)]
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
    read_error_: bool,

    // Offset of the file position indicator within the last block when an
    // EOF was detected.
    eof_offset_: usize,

    // Offset of the last record returned by ReadRecord.
    last_record_offset_: u64,

    // Offset of the first location past the end of buffer_.
    end_of_buffer_offset_: u64,
    initial_offset_: u64,
    log_number_: u64,
    recycled: bool,
    file_: SequentialFileReader<PosixSequentialFile>,
}

impl Reader {
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

    fn readPhysicalRecord(&mut self, fragment: &mut Vec<u8>, drop_size: &mut usize) -> isize {
        while (true) {}
        return 0;
    }

    // For kAbsoluteConsistency, on clean shutdown we don't expect any error
    // in the log files.  For other modes, we can ignore only incomplete records
    // in the last log file, which are presumably due to a write in progress
    // during restart (or from log recycling).
    //
    // TODO krad: Evaluate if we need to move to a more strict mode where we
    // restrict the inconsistency to only the last log
    fn readRecord(&mut self, record: &mut Vec<u8>, scratch: &mut Vec<u8>) -> bool {
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
        while true {
            let mut physical_record_offset = self.end_of_buffer_offset_ - self.buffer_.len() as u64;
            let mut drop_size: usize = 0;
            let record_type = self.readPhysicalRecord(&mut fragment, &mut drop_size);
        }
        return false;
    }

    //fn ReportDrop(bytes: usize, reason: state) {}
}
