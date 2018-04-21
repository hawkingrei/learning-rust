#[derive(Debug, PartialEq, Clone, Copy)]
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

pub const kMaxRecordType: u8 = RecordType::kRecyclableLastType as u8;
pub const kBlockSize: usize = 32768;
pub const kRecyclableHeaderSize: usize = 4 + 1 + 4 + 2;
pub const kHeaderSize: usize = 4 + 2 + 1;
