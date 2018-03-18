pub enum RecordType {
    kZeroType = 0,
    kFullType = 1,

    // For fragments
    kFirstType = 2,
    kMiddleType = 3,
    kLastType = 4,

    // For recycled log files
    kRecyclableFullType = 5,
    kRecyclableFirstType = 6,
    kRecyclableMiddleType = 7,
    kRecyclableLastType = 8,
}

pub static kMaxRecordType: u8 = RecordType::kRecyclableLastType as u8;
pub static kBlockSize: usize = 32768;
pub static kRecyclableHeaderSize: usize = 4 + 1 + 4 + 2;
pub static kHeaderSize: usize = 4 + 2 + 1;
