pub(crate) const DIRENT_ATTR_RO: u8 = 0x01;
pub(crate) const DIRENT_ATTR_HIDDEN: u8 = 0x02;
pub(crate) const DIRENT_ATTR_SYSTEM: u8 = 0x04;
pub(crate) const DIRENT_ATTR_VOLUME_LABEL: u8 = 0x08;
pub(crate) const DIRENT_ATTR_SUBDIR: u8 = 0x10;
pub(crate) const DIRENT_ATTR_ARCHIVE: u8 = 0x20;
pub(crate) const DIRENT_ATTR_DEVICE: u8 = 0x40;

pub(crate) const FILE_ATTR_CLOSED: u8 = 0;
pub(crate) const FILE_ATTR_FILE: u8 = 0x08;
pub(crate) const FILE_ATTR_ROOT: u8 = 0x40;
pub(crate) const FILE_ATTR_SUBDIR: u8 = 0x10;
pub(crate) const FILE_ATTR_DIRECTORY: u8 = FILE_ATTR_SUBDIR | FILE_ATTR_ROOT;

pub(crate) const FILE_FLAG_READ: u8 = 0x01;
pub(crate) const FILE_FLAG_WRITE: u8 = 0x02;
pub(crate) const FILE_FLAG_CONTIGUOUS: u8 = 0x40;
