mod constants;
mod data;
mod debug;
pub mod mbr;
pub mod partition;
pub mod volume;

use crate::sdcard::SdCardError;
pub use data::{
    DirEntry,
    File,
};
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
    VolumeIdMismatch,
    NotADirectory,
    InvalidPosition,
    ReadError,
    UnsupportedVersion,
    TooManySubdirs,
    Unknown,
}

impl From<SdCardError> for FatError {
    fn from(_: SdCardError) -> FatError {
        FatError::BlockDeviceFailed
    }
}
