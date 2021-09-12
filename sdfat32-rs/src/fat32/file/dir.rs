use super::{
    constants::*,
    File,
};
use crate::fat32::FatError;

struct FatDirEntry {
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
    file_size: u32,
}

pub struct DirectoryIterator<'a> {
    flags: u8,
    dir: &'a mut File,
    dir_cache: [u8; BYTES_PER_SECTOR],
}

impl<'a> Iterator for DirectoryIterator<'a> {
    type Item = Result<File, FatError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.dir.read_dir_entry();
        None
    }
}

impl<'a> File {
    fn read_dir_entry(&self) -> Result<FatDirEntry, FatError> {
        // Fat directory entries are on 32-byte boundaries
        if self.pos & 0x1f > 0 {
            return Err(FatError::InvalidPosition);
        }
        let i = (self.pos >> 5) & 0x0f;

        Ok(FatDirEntry {
            name: [0; 11],
            attributes: 0,
            case_flags: 0,
            creation_time_ms: 0,
            creation_time: 0,
            creation_date: 0,
            access_date: 0,
            first_cluster_high: 0,
            modify_time: 0,
            modify_date: 0,
            first_cluster_low: 0,
            file_size: 0,
        })
    }

    pub fn dir_next(&'a mut self) -> Result<DirectoryIterator<'a>, FatError> {
        if !self.is_directory() {
            return Err(FatError::NotADirectory);
        }

        Ok(DirectoryIterator {
            flags: 0,
            dir: self,
            dir_cache: [0; BYTES_PER_SECTOR],
        })
    }
}
