use super::constants::*;


// Directory entry attributes
const DIRENT_ATTR_RO: u8 = 0x01;
const DIRENT_ATTR_HIDDEN: u8 = 0x02;
const DIRENT_ATTR_SYSTEM: u8 = 0x04;
const DIRENT_ATTR_VOLUME_LABEL: u8 = 0x08;
const DIRENT_ATTR_SUBDIR: u8 = 0x10;
const DIRENT_ATTR_ARCHIVE: u8 = 0x20;
const DIRENT_ATTR_DEVICE: u8 = 0x40;
const DIRENT_ATTR_LONG_NAME: u8 = 0x0f;

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

const SELF_DIR: [u8; 11] = [DOT, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE];
const PARENT_DIR: [u8; 11] = [DOT, DOT, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE, SPACE];
const DELETED: u8 = 0xe5;

impl DirEntry {
    #[inline(always)]
    pub fn file_attributes(&self) -> u8 {
        // Attributes to pass on from the directory entry to the file object
        self.attributes & (DIRENT_ATTR_RO | DIRENT_ATTR_HIDDEN | DIRENT_ATTR_SYSTEM | DIRENT_ATTR_SUBDIR)
    }

    #[inline(always)]
    pub fn is_deleted(&self) -> bool {
        self.name[0] == DELETED
    }

    #[inline(always)]
    pub fn is_directory(&self) -> bool {
        self.attributes & DIRENT_ATTR_SUBDIR > 0
    }

    #[inline(always)]
    pub fn is_file_or_subdir(&self) -> bool {
        self.attributes & DIRENT_ATTR_VOLUME_LABEL == 0
    }

    #[inline(always)]
    pub fn is_long_name_component(&self) -> bool {
        self.attributes == DIRENT_ATTR_LONG_NAME
    }

    #[inline(always)]
    pub fn is_self_or_parent(&self) -> bool {
        self.name == SELF_DIR || self.name == PARENT_DIR
    }

    #[inline(always)]
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size
    }

    #[inline(always)]
    pub(crate) fn first_cluster(&self) -> u32 {
        ((self.first_cluster_high as u32) << 16) | (self.first_cluster_low as u32)
    }

    pub(crate) fn checksum(&self) -> u8 {
        let mut sum: u8 = 0;
        for c in self.name {
            sum = (((sum & 1) << 7) | (sum >> 1)) + c;
        }
        sum
    }
}
