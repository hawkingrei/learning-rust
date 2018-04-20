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
    read_error_: bool,
    eof_offset_: usize,
    last_record_offset_: u64,
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
            let skip_status = self.file_.Skip(block_start_location);
            if skip_status.is_ok() {
                //ReportDrop(static_cast<size_t>(block_start_location), skip_status);
                return false;
            }
        }
        return true;
    }

    //fn ReportDrop(bytes: usize, reason: state) {}
}
