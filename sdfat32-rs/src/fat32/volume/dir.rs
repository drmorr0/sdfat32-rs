use crate::{
    fat32::{
        FatError,
        File,
        Volume,
    },
    sdcard::SdCardRef,
};
use avr_hal_generic::port::PinOps;


#[derive(Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; 11],
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

impl DirEntry {
    fn new() -> DirEntry {
        DirEntry {
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
        }
    }
}

pub struct DirectoryIterator<'d, 'v: 'd, 's: 'v, CSPIN: PinOps> {
    flags: u8,
    dir: &'d mut File,
    sdcard: SdCardRef<'s, CSPIN>,
    vol: &'v mut Volume,
}

impl<CSPIN: PinOps> Iterator for DirectoryIterator<'_, '_, '_, CSPIN> {
    type Item = Result<DirEntry, FatError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Fat directory entries are on 32-byte boundaries
        if self.dir.pos & 0x1f > 0 {
            return Some(Err(FatError::InvalidPosition));
        }
        match self.vol.load_sector_for_file::<_, [DirEntry; 4]>(self.sdcard, self.dir) {
            Ok((entries_raw, sector_pos)) => {
                let entry_index = sector_pos >> 5;
                let entry = entries_raw.get()[entry_index];
                self.dir.pos += 32;

                if entry.name[0] == 0x0 {
                    None
                } else {
                    Some(Ok(entry))
                }
            },
            Err(e) => Some(Err(e)),
        }
    }
}

impl Volume {
    pub fn dir_next<'d, 'v: 'd, 's: 'v, CSPIN: PinOps>(
        &'v mut self,
        sdcard: SdCardRef<'s, CSPIN>,
        dir: &'d mut File,
    ) -> Result<DirectoryIterator<'s, 'd, 'v, CSPIN>, FatError> {
        if !dir.is_directory() {
            return Err(FatError::NotADirectory);
        }

        Ok(DirectoryIterator { flags: 0, dir, sdcard, vol: self })
    }
}
