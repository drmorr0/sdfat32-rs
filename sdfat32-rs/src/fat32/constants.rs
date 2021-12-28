use crate::sdcard::BLOCK_SIZE;

pub const DIR_SEPARATOR: u8 = '/' as u8;
pub const MAX_PATHNAME_LEN: usize = 256;
pub const MAX_LFN_LEN: usize = 255;

pub(crate) const SPACE: u8 = ' ' as u8;
pub(crate) const DOT: u8 = '.' as u8;

pub(crate) const BYTES_PER_SECTOR: usize = BLOCK_SIZE;
pub(crate) const LOG2_BYTES_PER_SECTOR: u8 = 9;
pub(crate) const ROOT_CLUSTER: u32 = 2;
pub(crate) const SECTOR_MASK: u16 = 0x1FF;

pub(crate) type SECTOR = [u8; BYTES_PER_SECTOR];

// File attributes
pub(crate) const ATTR_CLOSED: u8 = 0;
pub(crate) const ATTR_FILE: u8 = 0x08;
pub(crate) const ATTR_ROOT: u8 = 0x40;
pub(crate) const ATTR_SUBDIR: u8 = 0x10;
pub(crate) const ATTR_DIRECTORY: u8 = ATTR_SUBDIR | ATTR_ROOT;

pub(crate) const FLAG_READ: u8 = 0x01;
pub(crate) const FLAG_WRITE: u8 = 0x02;
pub(crate) const FLAG_CONTIGUOUS: u8 = 0x40;

// API constants
pub const O_RDONLY: u8 = 0x0;
pub const O_WRONLY: u8 = 0x1;
pub const O_RDWR: u8 = 0x2;
pub const O_AT_END: u8 = 0x4;
pub const O_APPEND: u8 = 0x8;
pub const O_CREAT: u8 = 0x10;
pub const O_TRUNC: u8 = 0x20;
pub const O_EXCL: u8 = 0x40;
pub const O_SYNC: u8 = 0x80;
