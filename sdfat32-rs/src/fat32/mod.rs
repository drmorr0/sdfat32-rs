mod debug;
pub mod file;
pub mod mbr;
pub mod partition;
pub mod volume;

use crate::sdcard::SdCardError;
pub use file::File;
pub use mbr::Mbr;
pub use volume::Volume;

pub enum FatError {
    BlockDeviceFailed = 1,
    CorruptMBR,
    BadPartitionNumber,
    CorruptPartition,
    FileClosed,
    SeekError,
    InvalidCluster,
    CorruptFat,
    NotADirectory,
    UnsupportedVersion,
    Unknown,
}

impl From<SdCardError> for FatError {
    fn from(_: SdCardError) -> FatError {
        FatError::BlockDeviceFailed
    }
}
