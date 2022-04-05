pub mod constants;
mod debug;
mod dir_entry;
mod file;
mod mbr;
mod partition;
mod volume;

use crate::sdcard::SdCardError;
pub use dir_entry::DirEntry;
pub use file::File;
pub use mbr::Mbr;
pub use partition::Partition;
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
    VolumeIdMismatch,
    NotADirectory,
    InvalidPosition,
    ReadError,
    UnsupportedVersion,
    TooManySubdirs,
    LfnParseError,
    ParsePathError,
    FileNotFound,
    Unknown,
}

impl From<SdCardError> for FatError {
    fn from(_: SdCardError) -> FatError {
        FatError::BlockDeviceFailed
    }
}
