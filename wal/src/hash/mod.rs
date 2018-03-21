mod crc16;
mod crc64;
mod crc32;

pub use self::crc32::crc32;
// used for protecting large snapshot files
pub use self::crc64::crc64;

// used for protecting individual log entries
pub use self::crc16::{crc16, crc16_arr};
