use super::{
    constants::*,
    DirEntry,
};
use crate::fat32::constants::*;


// Clusters can't be usize because the FS address space is larger than 16 bits
#[derive(Clone, Copy)]
pub struct File {
    pub(crate) cluster: u32,
    pub(crate) pos: u32,
    pub(crate) start_cluster: u32,
    pub(crate) vol_id: u8,
    attributes: u8,
    flags: u8,
    size: u32,
}

impl File {
    pub fn empty() -> File {
        File {
            cluster: 0,
            pos: 0,
            start_cluster: 0,
            vol_id: 123,
            attributes: 0,
            flags: 0,
            size: 0,
        }
    }

    pub(crate) fn open(vol_id: u8, entry: &DirEntry) -> File {
        File {
            cluster: entry.first_cluster(),
            pos: 0,
            start_cluster: entry.first_cluster(),
            vol_id,
            attributes: entry.file_attributes(),
            flags: FILE_FLAG_READ, // TODO
            size: entry.size(),
        }
    }

    pub(crate) fn open_root(vol_id: u8) -> File {
        File {
            cluster: ROOT_CLUSTER,
            pos: 0,
            start_cluster: ROOT_CLUSTER,
            vol_id,
            attributes: FILE_ATTR_ROOT,
            flags: FILE_FLAG_READ, // TODO
            size: 0,
        }
    }

    #[inline(always)]
    pub fn is_contiguous(&self) -> bool {
        self.flags & FILE_FLAG_CONTIGUOUS > 0
    }

    #[inline(always)]
    pub fn is_directory(&self) -> bool {
        self.attributes & FILE_ATTR_DIRECTORY > 0
    }

    #[inline(always)]
    pub fn is_file(&self) -> bool {
        self.attributes & FILE_ATTR_FILE > 0
    }

    #[inline(always)]
    pub fn is_open(&self) -> bool {
        self.attributes > 0
    }

    #[inline(always)]
    pub fn is_readable(&self) -> bool {
        self.flags & FILE_FLAG_READ > 0
    }

    #[inline(always)]
    pub fn is_root(&self) -> bool {
        self.attributes & FILE_ATTR_ROOT > 0
    }

    #[inline(always)]
    pub fn size(&self) -> u32 {
        self.size
    }
}
