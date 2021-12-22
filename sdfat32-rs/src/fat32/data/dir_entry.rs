use super::constants::*;


#[derive(Clone, Copy)]
pub struct DirEntry {
    name: [u8; 11],
    attributes: u8,
    case_flags: u8,
    creation_time_ms: u8,
    creation_time: u16,
    creation_date: u16,
    access_date: u16,
    first_cluster_high: u16,
    modify_time: u16,
    modify_date: u16,
    first_cluster_low: u16,
    size: u32,
}

// b".          "
const SELF_DIR: [u8; 11] = [0x2e, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20];
// b"..         "
const PARENT_DIR: [u8; 11] = [0x2e, 0x2e, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20];

impl DirEntry {
    #[inline(always)]
    pub fn file_attributes(&self) -> u8 {
        // Attributes to pass on from the directory entry to the file object
        self.attributes & (DIRENT_ATTR_RO | DIRENT_ATTR_HIDDEN | DIRENT_ATTR_SYSTEM | DIRENT_ATTR_SUBDIR)
    }

    #[inline(always)]
    pub fn is_directory(&self) -> bool {
        self.attributes & DIRENT_ATTR_SUBDIR > 0
    }

    #[inline(always)]
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    #[inline(always)]
    pub fn is_self_or_parent(&self) -> bool {
        self.name == SELF_DIR || self.name == PARENT_DIR
    }

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    pub(crate) fn first_cluster(&self) -> u32 {
        ((self.first_cluster_high as u32) << 16) | (self.first_cluster_low as u32)
    }
}
