use wal;
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
    initial_offset: u64,
    log_number_: u64,
    recycled: bool,
}
