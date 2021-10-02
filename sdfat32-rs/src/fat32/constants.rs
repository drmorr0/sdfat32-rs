use crate::sdcard::BLOCK_SIZE;

pub(crate) const ROOT_CLUSTER: u32 = 2;
pub(crate) const BYTES_PER_SECTOR: usize = BLOCK_SIZE;
pub(crate) const LOG2_BYTES_PER_SECTOR: u8 = 9;
pub(crate) const SECTOR_MASK: u16 = 0x1FF;
