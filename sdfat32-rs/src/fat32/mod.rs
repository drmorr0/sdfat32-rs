pub mod mbr;
pub mod partition;

pub use mbr::Mbr;
pub use partition::Partition;

pub enum FatError {
    CorruptMBR = 1,
    BadPartitionNumber,
    CorruptPartition,
    Unsupported,
    Unknown,
}
