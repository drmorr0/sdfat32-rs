mod debug;
pub mod file;
pub mod mbr;
pub mod partition;
pub mod volume;

pub use file::File;
pub use mbr::Mbr;
pub use volume::Volume;

pub enum FatError {
    CorruptMBR = 1,
    BadPartitionNumber,
    CorruptPartition,
    Unsupported,
    FileClosed,
    SeekError,
    Unknown,
}
