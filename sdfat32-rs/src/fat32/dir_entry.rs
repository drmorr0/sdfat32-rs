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

pub enum DirEntry {
    Long(LFN, usize, u8),
    Short(SFN, bool),
}

impl DirEntry {
    #[inline(always)]
    pub fn is_self_or_parent(&self) -> bool {
        match self {
            DirEntry::Long(..) => false,
            DirEntry::Short(sfn, _) => sfn.is_self_or_parent(),
        }
    }

    #[inline(always)]
    pub fn is_deleted(&self) -> bool {
        match self {
            DirEntry::Long(lfn, ..) => lfn.sequence_byte == DELETED,
            DirEntry::Short(sfn, _) => sfn.name[0] == DELETED,
        }
    }

    #[inline(always)]
    pub fn is_hidden(&self) -> bool {
        match self {
            DirEntry::Long(_, _, attr) => attr & DIRENT_ATTR_HIDDEN > 0,
            DirEntry::Short(sfn, _) => sfn.is_hidden(),
        }
    }
}

pub struct LFN {
    sequence_byte: u8,
    unicode1: [u8; 10],
    _always_0x0f: u8,
    _always_zero_1: u8,
    checksum: u8,
    unicode2: [u8; 12],
    _always_zero_2: [u8; 2],
    unicode3: [u8; 4],
}

impl LFN {
    pub fn get_char(&self, i: usize) -> u8 {
        if i < 5 {
            self.unicode1[2 * i]
        } else if i < 11 {
            self.unicode2[2 * i - 10]
        } else if i < 13 {
            self.unicode3[2 * i - 22]
        } else {
            0
        }
    }

    #[inline(always)]
    pub(crate) fn checksum(&self) -> u8 {
        self.checksum
    }

    #[inline(always)]
    pub(crate) fn sequence_num(&self) -> usize {
        (self.sequence_byte & 0x1f) as usize
    }

    #[inline(always)]
    pub(crate) fn is_last_in_sequence(&self) -> bool {
        self.sequence_byte & 0x40 > 0
    }
}

#[derive(Clone, Copy)]
pub struct SFN {
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

impl SFN {
    #[inline(always)]
    pub fn file_attributes(&self) -> u8 {
        // Attributes to pass on from the directory entry to the file object
        self.attributes & (DIRENT_ATTR_RO | DIRENT_ATTR_HIDDEN | DIRENT_ATTR_SYSTEM | DIRENT_ATTR_SUBDIR)
    }

    #[inline(always)]
    pub fn is_hidden(&self) -> bool {
        self.attributes & DIRENT_ATTR_HIDDEN > 0
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
